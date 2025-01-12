
use std::{fmt::Display, fs::File, io::Read, ops::{Index, IndexMut}};

use iced::{
    widget::{button, container, text, text_input, Column, ComboBox, Row},
    Element, Length::{self, Fill}
};
use iced::widget::combo_box::State as ComboState;
use rfd::FileDialog;
use crate::state::Message;

#[derive(Clone)]
pub struct PacketField{
    index : usize,
    combo_state : ComboState<PacketDataType>,
    datatype : Option<PacketDataType>,
    data_string : String,

}
#[derive(Debug, Clone, Copy)]
pub enum PacketDataType{
    Bytes,
    CStr,
    Utf8Str,
    U64,
    U32,
    U16,
    U8,
    I64,
    I32,
    I16,
    I8
}

impl Display for PacketDataType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",  self)
    }
}


#[derive(Debug, Clone)]
pub enum PVMessage{
    DataEntry(String, usize),
    DataType(PacketDataType, usize),
    AddField,
    RemoveField(usize),
    OpenFile(usize)
}


#[derive(Default,Clone)]
pub struct PacketView{
    index : usize,
    lable : String,
    fields : Vec<PacketField>
}

impl PacketView{
    pub fn new(index :usize) -> Self{
        Self { index, lable: Default::default(), fields: Default::default() }
    }
    pub fn update(&mut self, msg : PVMessage){
        match msg {
            PVMessage::DataEntry(dat, i) =>{
                self[i].data_string = dat
            },
            PVMessage::AddField =>{
                self.add_field();
            },
            PVMessage::DataType(x, i) =>{
                self[i].datatype = Some(x);
                
            },
            PVMessage::RemoveField(x) => {
                self.fields.remove(x);
            }
            PVMessage::OpenFile(x) => self[x].data_string = FileDialog::new().pick_file().unwrap_or_default().as_path().to_str().unwrap().to_string(),
            //_ => ()
        }
    }
    pub fn draw(&self) -> Element<'_, Message>{
        let mut col = Column::new();
        col = col.push(
            text!("{}", self.lable)
        );
        col = col.push(
            button("Add field").on_press(Message::PVMessage(self.index, PVMessage::AddField))
        );
        col = col.push(
            button("Remove Packet").on_press(Message::RemovePacket(self.index))
        );
        for c in &self.fields{
            col = col.push(c.draw(self.index));
        }
        col = col.push(
            button("Send packet").on_press(Message::SendPacket(self.index))
        );
        col.into()
    }
    pub fn add_field(&mut self){
        self.fields.push(PacketField::new(self.fields.len()));
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        self.fields.iter().map(|x|x.to_bytes()).flatten().collect()
    }
}

impl IndexMut<usize> for PacketView{
    fn index_mut(&mut self, index: usize) -> &mut PacketField {
        &mut self.fields[index]
    }
}
impl Index<usize> for PacketView{

    fn index(&self, index: usize) -> &PacketField {
        &self.fields[index]
    }
    
    type Output = PacketField;
}


impl PacketField{
    pub(self) fn new(index : usize) -> Self{
        Self { 
            index,
            combo_state: ComboState::new(
                vec![
                        PacketDataType::Bytes,
                        PacketDataType::CStr,
                        PacketDataType::Utf8Str,
                        PacketDataType::U64,
                        PacketDataType::U32,
                        PacketDataType::U16,
                        PacketDataType::U8,
                        PacketDataType::I64,
                        PacketDataType::I32,
                        PacketDataType::I16,
                        PacketDataType::I8
                    ]
                ),
                data_string: Default::default(),
                datatype: None
            }
        }
    pub(self) fn to_bytes(&self) -> Vec<u8>{
        if let Some(dat) = self.datatype{
            match dat{
                PacketDataType::Bytes => {
                    let mut f = File::open(self.data_string.clone()).expect("File Not found");
                    let mut ret : Vec<u8> = Default::default();
                    f.read_to_end(&mut ret).unwrap();
                    ret
                },
                PacketDataType::CStr => std::ffi::CString::new(self.data_string.as_str()).unwrap().as_bytes().iter().cloned().collect(),
                PacketDataType::Utf8Str => self.data_string.as_bytes().iter().cloned().collect(),
                PacketDataType::U64 => self.data_string.parse::<u64>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::U32 => self.data_string.parse::<u32>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::U16 => self.data_string.parse::<u16>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::U8  =>  self.data_string.parse::<u8>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I64 => self.data_string.parse::<i64>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I32 => self.data_string.parse::<i32>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I16 => self.data_string.parse::<i16>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I8  =>  self.data_string.parse::<i8>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
            }
        }
        else{
            Default::default()
        }
    } 

    pub fn draw(&self, parent_index : usize) -> Element<'_, Message>{
        let mut row = Row::new();
        let idx = self.index.clone();
        row = row.push(
            ComboBox::new(
                &self.combo_state, "Please select a data type", self.datatype.as_ref(), 
            move|x|
                {
                    Message::PVMessage(parent_index, PVMessage::DataType(x, idx))
                }
            ).width(Length::FillPortion(1))
        );

        if let Some(PacketDataType::Bytes) = self.datatype{
            row = row.push(
                button("Select a file")
                .on_press(Message::PVMessage(parent_index, PVMessage::OpenFile(self.index)))
            );
        }
        else{
            let p2 = parent_index.clone();
            row = row.push(
                text_input(
                    "Enter data here",
                    &self.data_string
                ).on_input(move |x|{Message::PVMessage(p2, PVMessage::DataEntry(x, self.index))}).width(Length::FillPortion(3)),
            );
        }
        row = row.push(
            button("remove field")
                .on_press(Message::PVMessage(parent_index, PVMessage::RemoveField(self.index)))
                .padding(10)
        );


        container(
            row
        )
        .padding(10)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }
}