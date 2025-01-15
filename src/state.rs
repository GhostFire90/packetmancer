use std::{fs::{read_to_string, File}, io::{Read, Write}, ops::{Index, IndexMut}};

use iced::{widget::{button, row, text, text_input, Column}, Element};
use rfd::FileDialog;

use crate::packet::{PVMessage, PacketView, PacketDataType};
use std::net::TcpStream;


#[derive(Debug, Clone)]
pub enum Message{
    AddPacket,
    RemovePacket(usize),
    PVMessage(usize, PVMessage),
    IpEntry(String),  
    PortEntry(String),
    Connect,
    Disconnect,
    SendPacket(usize),
    RecievePacket(usize),
    OpenPacket
}

#[derive(Default)]
pub struct State{
    packet_views : Vec<PacketView>,
    current_ip : String,
    current_port : String,
    sock : Option<TcpStream>,
}

impl State{
    pub fn update(&mut self, msg : Message){
        match msg{
            Message::PVMessage(i, x) => self[i].update(x),
            Message::AddPacket => self.add_packet(),
            Message::IpEntry(x) => self.current_ip = x,
            Message::PortEntry(x) => self.current_port = x,
            Message::Connect => self.connect().unwrap(),
            Message::Disconnect => self.disconnect(),
            Message::RemovePacket(x) => {
                self.packet_views.remove(x);
                self.packet_views.iter_mut().enumerate().for_each(|(i,x)| x.index = i);
            },
            Message::SendPacket(x) => self.send(&self.packet_views[x].clone()).unwrap(),
            Message::OpenPacket => {
                let fpath = FileDialog::new().add_filter("json", &["json"]).pick_file().unwrap();
                let fstr = read_to_string(fpath).unwrap();
                let obj = jzon::parse(&fstr).unwrap();
                self.packet_views.push(PacketView::from(obj));
            }
            Message::RecievePacket(x) => self.recieve(x).unwrap(),
            
        };
    }
    pub fn add_packet(&mut self){
        self.packet_views.push(PacketView::new(self.packet_views.len()));
    }
    pub fn draw(&self) -> Element<Message>{
        let mut col = Column::new();
        col = col.push(
            row(
                [
                    button("New Packet").on_press(Message::AddPacket).into(),
                    button("Open Packet").on_press(Message::OpenPacket).into(),
                    
                ]
            ).spacing(5)
        );
        let conn_row = {
            if self.sock.is_none() {
                row(
                    [
                            button("Connect").on_press(Message::Connect).into(),
                        ]
                    )
            }
            else {
                row(
                    [
                            button("Disconnect").on_press(Message::Disconnect).into(),
                            text!("Connected to: {}:{}", self.current_ip, self.current_port).into()
                        ]
                    ).spacing(5)
            }
        };

        col = col.push(
            conn_row
        );
        col = col.push(
            row(
                [
                    text_input("Input IP", &self.current_ip).on_input(Message::IpEntry).into(),
                    text_input("Input Port", &self.current_port).on_input(Message::PortEntry).into()
                ]
            ).spacing(5)
        );
        for v in &self.packet_views{
            col = col.push(v.draw());
        }
        col.spacing(10).into()
    }
    fn disconnect(&mut self){
        if let Some(s) = &self.sock{
            s.shutdown(std::net::Shutdown::Both).unwrap();
            self.sock = None
        }
    }

    fn connect(&mut self) -> std::io::Result<()>{
        self.disconnect();
        self.sock = Some(TcpStream::connect(format!("{}:{}", self.current_ip, self.current_port))?);
        Ok(())
    }

    fn send(&mut self, packet : &PacketView) -> std::io::Result<()>{
        if let Some(s) = &mut self.sock{
            s.write(&packet.to_bytes())?;
        }
        Ok(())
    }

    fn recieve(&mut self, p_idx : usize) -> std::io::Result<()>{
        let packet = &mut self.packet_views[p_idx];
        if let Some(s) = &mut self.sock{
            for i in 0..packet.fields.len(){
                match packet.fields[i].datatype.unwrap(){
                    PacketDataType::Bytes(sizing_method) => {
                        let fdiag = FileDialog::new()
                        .add_filter("binary", &["bin",""])
                        .save_file();
                        if let Some(fpath) = fdiag{
                            let mut f = File::create(fpath.clone())?;
                            let mut rsize = 
                                match sizing_method {
                                    crate::packet::SizingMethod::SizeHeader(x) => packet.get_field(x).data_string.parse::<usize>().unwrap_or_default(),
                                    crate::packet::SizingMethod::FixedSize(x) => x,
                                };
                            let mut dat = [0;4096];
                            while rsize > 0{
                                let count = s.read(&mut dat)?;
                                println!("{count}");
                                rsize -= count;
                                f.write(&dat[0..count])?;
                            }
                        }
                    },
                    PacketDataType::CStr => {
                        let mut dat : Vec<u8> = Vec::new();
                        let mut c : [u8;1] = [0];
                        while s.read(&mut c)? > 0 && c[0] != 0{
                            dat.push(c[0]);
                        }
                        dat.push(0);
                        packet.fields[i].data_string = std::ffi::CString::from_vec_with_nul(dat).unwrap().to_str().unwrap().to_string();
                    },
                    _ => {
                        let dtype = packet.fields[i].datatype.unwrap();
                        let mut dat : Vec<u8> = Vec::with_capacity(dtype.data_size());
                        s.read(&mut dat)?;
                        packet.fields[i].data_string = dtype.bytes_to_val(dat).as_ref().to_string();
                    }
                }
            }
        } 
        Ok(())
    }


}

impl Index<usize> for State{
    type Output = PacketView;

    fn index(&self, index: usize) -> &Self::Output {
        &self.packet_views[index]
    }
}
impl IndexMut<usize> for State{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.packet_views[index]
    }
}