use std::{io::Write, ops::{Index, IndexMut}};

use iced::{widget::{button, row, text_input, Column}, Element};

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
    SendPacket(usize)
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
            Message::RemovePacket(x) => {self.packet_views.remove(x);},
            Message::SendPacket(x) => self.send(&self.packet_views[x].clone()).unwrap(),
            
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
                    button("Connect").on_press(Message::Connect).into(),
                    button("Disconnect").on_press(Message::Disconnect).into()
                ]
            )
        );
        col = col.push(
            row(
                [
                    text_input("Input IP", &self.current_ip).on_input(Message::IpEntry).into(),
                    text_input("Input Port", &self.current_port).on_input(Message::PortEntry).into()
                ]
            )
        );
        for v in &self.packet_views{
            col = col.push(v.draw());
        }
        col.into()
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