
use std::{fmt::Display, fs::File, io::{Read, Write}, ops::{Index, IndexMut}};

use iced::{
    widget::{button, container, row, text, text_input, Column, ComboBox, Row},
    Element, Length::{self, Fill}
};
use iced::widget::combo_box::State as ComboState;
use jzon::{object, JsonValue};
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
    Bytes(usize),
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
    OpenFile(usize),
    SavePacket,
}


#[derive(Default,Clone)]
pub struct PacketView{
    pub(crate) index : usize,
    lable : String,
    fields : Vec<PacketField>
}

impl Into<JsonValue> for PacketDataType {
    fn into(self) -> JsonValue {
        match self {
            Self::Bytes(x) => object! {
                size: x
            },
            _ => JsonValue::String(format!("{}", self))            
        }
    }
}
impl From<JsonValue> for PacketDataType{
    fn from(value: JsonValue) -> Self {
        if let Some(s) = value.as_str(){
            match s{
                "CStr" => Self::CStr,
                "Utf8Str" => Self::Utf8Str,
                "U64" => Self::U64,
                "U32" => Self::U32,
                "U16" => Self::U16,
                "U8" => Self::U8,
                "I64" => Self::I64,
                "I32" => Self::I32,
                "I16" => Self::I16,
                "I8" => Self::I8,
                _ => panic!("Unexpected PacketDataType")
            }
        }
        else if let Some(obj) = value.as_object(){
            Self::Bytes(obj["size"].as_usize().unwrap())
        }
        else {
            panic!("Unexpected PacketDataType")
        }
        

    }
}

impl Into<JsonValue> for PacketView{
    fn into(self) -> JsonValue {
        object! {
            index: self.index,
            lable: self.lable,
            fields: self.fields
        }
    }
}
impl Into<JsonValue> for PacketField{
    fn into(self) -> JsonValue {
        object! {
            index: self.index,
            datatype : self.datatype,
            data_string : self.data_string
        }
    }
}

impl From<JsonValue> for PacketField{
    fn from(value: JsonValue) -> Self {
        let idx = value["index"].as_usize().unwrap();
        let dattype : PacketDataType = value["datatype"].clone().into();
        let dat_str = value["data_string"].as_str().unwrap();
        Self { index: idx, combo_state: Self::create_combo(), datatype: Some(dattype), data_string: dat_str.to_string() }
    }
}

impl From<JsonValue> for PacketView{
    fn from(value: JsonValue) -> Self {
        let index = value["index"].as_usize().unwrap();
        let lable = value["lable"].as_str().unwrap().to_string();
        let fields : Vec<PacketField> = value["fields"].as_array().unwrap().iter().map(|x| PacketField::from(x.clone())).collect();
        Self { index, lable, fields}
    }
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
                self.fields.iter_mut().enumerate().for_each(|(i,x)| x.index = i);
            }
            PVMessage::OpenFile(x) => self[x].data_string = FileDialog::new().pick_file().unwrap_or_default().as_path().to_str().unwrap().to_string(),
            PVMessage::SavePacket => {
                let fpath = FileDialog::new().add_filter("json", &["json"]).save_file().unwrap();
                let mut f = File::create(fpath).unwrap();
                f.write(jzon::stringify(self.clone()).as_bytes()).unwrap();
            },


            //_ => ()
        }
    }
    pub fn draw(&self) -> Element<'_, Message>{
        let mut col = Column::new();
        col = col.push(
            text!("{}", self.lable)
        );
        col = col.push(
            row![
                button("Add field").on_press(Message::PVMessage(self.index, PVMessage::AddField)),
                button("Save Packet").on_press(Message::PVMessage(self.index, PVMessage::SavePacket))
            ].spacing(5)
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
        col.spacing(10).into()
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
            combo_state: Self::create_combo(),
                data_string: Default::default(),
                datatype: None
            }
        }
    pub(self) fn to_bytes(&self) -> Vec<u8>{
        if let Some(dat) = self.datatype{
            match dat{
                PacketDataType::Bytes(_) => {
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

    fn create_combo() -> ComboState<PacketDataType>{
        ComboState::new(
            vec![
                    PacketDataType::Bytes(0),
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
            )
    }

    pub fn draw(&self, parent_index : usize) -> Element<'_, Message>{
        let mut row = Row::new();
        let idx = self.index.clone();
        row = row.push(text::Text::new(format!("{}", self.index)));
        row = row.push(
            ComboBox::new(
                &self.combo_state, "Please select a data type", self.datatype.as_ref(), 
            move|x|
                {
                    Message::PVMessage(parent_index, PVMessage::DataType(x, idx))
                }
            ).width(Length::FillPortion(1))
        );

        if let Some(PacketDataType::Bytes(_)) = self.datatype{
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
            row.spacing(5)
        )
        .padding(10)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }
}