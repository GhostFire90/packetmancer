use std::{fs::read_to_string, io::Write, ops::{Index, IndexMut}};

use iced::{widget::{button, row, text, text_input, Column}, Element};
use rfd::FileDialog;

use crate::packet::{PVMessage, PacketView};
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
    OpenPacket
}

#[derive(Default)]
pub struct State{
    packet_views : Vec<PacketView>,
    current_ip : String,
    current_port : String,
    sock : Option<TcpStream>
    
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