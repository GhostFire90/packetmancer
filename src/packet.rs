
use std::{fmt::{Debug, Display}, fs::{metadata, File}, io::{Read, Write}, ops::{Index, IndexMut}};

use iced::{
    widget::{button, combo_box, container, row, text, text_input, toggler, Column, ComboBox, Row},
    Element, Length::{self, Fill}
};
use iced::widget::combo_box::State as ComboState;
use jzon::{object, JsonValue};
use rfd::FileDialog;
use crate::state::Message;

#[derive(Clone)]
pub struct PacketField{
    pub(crate)index : usize,
    pub(crate)dtype_combo_state : ComboState<PacketDataType>,
    pub(crate)datatype : Option<PacketDataType>,
    pub(crate) data_string : String,
    pub(crate)smethod_combo_state : ComboState<SizingMethod>,
    pub(crate)sizing_method : Option<SizingMethod>,
    pub(crate)sizing_meth_str : String

}
#[derive(Debug, Clone, Copy)]
pub enum PacketDataType{
    Bytes(SizingMethod),
    CStr,
    U64,
    U32,
    U16,
    U8,
    I64,
    I32,
    I16,
    I8
}

impl PacketDataType{
    pub const fn data_size(&self) -> usize{
        match self {
            PacketDataType::Bytes(_) => panic!("Use the sizing method"),
            PacketDataType::CStr => panic!("CStrings are arbitrarily sized"),
            PacketDataType::U64 | PacketDataType::I64 => 8,
            PacketDataType::U32 | PacketDataType::I32 => 4,
            PacketDataType::U16 | PacketDataType::I16 => 2,
            PacketDataType::U8  | PacketDataType::I8  => 1,
        }
    }
    pub fn bytes_to_val(&self, dat : Vec<u8>) -> Box<dyn ToString>{
        match self{
            PacketDataType::Bytes(_) => panic!("Not for this"),
            PacketDataType::CStr => panic!("Not for this"),
            PacketDataType::U64 => Box::new(u64::from_ne_bytes(dat.try_into().unwrap())),
            PacketDataType::U32 => Box::new(u32::from_ne_bytes(dat.try_into().unwrap())),
            PacketDataType::U16 => Box::new(u16::from_ne_bytes(dat.try_into().unwrap())),
            PacketDataType::U8  => Box::new(u8::from_ne_bytes(dat.try_into().unwrap())),
            PacketDataType::I64 => Box::new(i64::from_ne_bytes(dat.try_into().unwrap())),
            PacketDataType::I32 => Box::new(i32::from_ne_bytes(dat.try_into().unwrap())),
            PacketDataType::I16 => Box::new(i16::from_ne_bytes(dat.try_into().unwrap())),
            PacketDataType::I8  => Box::new(i8::from_ne_bytes(dat.try_into().unwrap())),
        }
    }
}

impl Display for PacketDataType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Bytes(_) = self{
            write!(f, "Bytes")
        }
        else{
            write!(f, "{:?}",  self)
        }
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
    ToggleRecieve(bool),
    ChangeSizingMethod(SizingMethod, usize),
    MethodEntry(String, usize)
}


#[derive(Default,Clone)]
pub struct PacketView{
    pub(crate) index : usize,
    lable : String,
    recieve : bool,
    pub(crate) fields : Vec<PacketField>
}


#[derive(Debug, Clone, Copy)]
pub enum SizingMethod{
    SizeHeader(usize), //index of field
    FixedSize(usize),  //fixed size
}

impl SizingMethod{
    pub(self) fn update(&self, view : &mut PacketView, field : &PacketField) {
        match self{
            SizingMethod::FixedSize(_) => (),
            SizingMethod::SizeHeader(x) => {
                if !view.recieve{
                    let f = metadata(field.data_string.clone());
                    if let Ok(met) = f{
                        //println!("Got to meta_data");
                        view[*x].data_string = met.len().to_string();
                    }
                    else if let Err(e) = f{
                        println!("Couldnt get metadata for field {x} {}: {:?}",&field.data_string, e)
                    }
                }

            }
        }
    }
}
impl Into<JsonValue> for SizingMethod{
    fn into(self) -> JsonValue {
        match self {
            SizingMethod::SizeHeader(x) | SizingMethod::FixedSize(x) => {
                object! {
                    method : self.to_string(),
                    size   : x
                }
            },
            
        }
    }
}
impl From<JsonValue> for SizingMethod{
    fn from(value: JsonValue) -> Self {
        let method = value["method"].as_str().unwrap();
        match method{
            "SizeHeader" => {
                Self::SizeHeader(value["size"].as_usize().unwrap())
            }
            "FixedSize" =>{
                Self::FixedSize(value["size"].as_usize().unwrap())
            }
            _ => panic!("Unknown Sizing Method")
        }
    }
}
impl Display for SizingMethod{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizingMethod::SizeHeader(_) => write!(f, "SizeHeader"),
            SizingMethod::FixedSize(_) => write!(f, "FixedSize"),
        }
    }
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
            Self::Bytes(obj["sizing_method"].clone().into())
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
            recieve: self.recieve,            
            lable: self.lable,
            fields: self.fields
        }
    }
}
impl Into<JsonValue> for PacketField{
    fn into(self) -> JsonValue {
        object! {
            index : self.index,
            sizing_method : self.sizing_method,
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
        let meth : SizingMethod = value["sizing_method"].clone().into();
        Self { 
                index: idx, 
                dtype_combo_state: Self::create_dtype_combo(),
                datatype: Some(dattype), 
                data_string: dat_str.to_string(),
                smethod_combo_state: Self::create_smeth_combo(),
                sizing_method: Some(meth),
                sizing_meth_str : match meth{
                    SizingMethod::FixedSize(x) | SizingMethod::SizeHeader(x) => x.to_string()
                }
            }
    }
}

impl From<JsonValue> for PacketView{
    fn from(value: JsonValue) -> Self {
        let index = value["index"].as_usize().unwrap();
        let lable = value["lable"].as_str().unwrap().to_string();
        let fields : Vec<PacketField> = value["fields"].as_array().unwrap().iter().map(|x| PacketField::from(x.clone())).collect();
        let recieve = value["recieve"].as_bool().unwrap();
        Self { index, recieve, lable, fields}
    }
}


impl PacketView{
    pub fn new(index :usize) -> Self{
        Self { index, recieve: false, lable: Default::default(), fields: Default::default() }
    }

    pub fn get_field(&self, index : usize) -> PacketField{
        self.fields[index].clone()
    }

    pub fn update(&mut self, msg : PVMessage){
        match msg {
            PVMessage::DataEntry(dat, i) =>{
                if self[i].is_valid_entry(&dat){
                    self[i].data_string = dat
                }
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
            PVMessage::ToggleRecieve(x) => self.recieve = x,
            PVMessage::ChangeSizingMethod(sizing_method, x) => {
                    match sizing_method{
                        SizingMethod::SizeHeader(_) =>{
                            if self.fields.len() > 1{
                                self[x].sizing_method = Some(sizing_method);
                                self[x].sizing_method.unwrap().update(self, &self[x].clone());
                            }
                        }
                        _ => {
                            self[x].sizing_method = Some(sizing_method);
                            self[x].sizing_method.unwrap().update(self, &self[x].clone());
                        }
                    }
                },
                PVMessage::MethodEntry(s, x) => {
                    if let Some(SizingMethod::SizeHeader(_)) = &self[x].sizing_method{
                        if x < self[x].index{
                            self[x].sizing_meth_str = s;
                            self[x].sizing_method.unwrap().update(self, &self[x].clone());
                        }
                    } 
                    else{
                        self[x].sizing_meth_str = s;
                        self[x].sizing_method.unwrap().update(self, &self[x].clone());
                    }
                }

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
                button("Save Packet").on_press(Message::PVMessage(self.index, PVMessage::SavePacket)),
                toggler(self.recieve).on_toggle(|x| Message::PVMessage(self.index, PVMessage::ToggleRecieve(x)))
            ].spacing(5)
        );
        col = col.push(
            button("Remove Packet").on_press(Message::RemovePacket(self.index))
        );
        for c in &self.fields{
            col = col.push(c.draw(self.index));
        }
        col = col.push(
            if !self.recieve{
                button("Send packet").on_press(Message::SendPacket(self.index))
            }
            else{
                button("Recieve packet").on_press(Message::RecievePacket(self.index))
            }
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
                dtype_combo_state: Self::create_dtype_combo(),
                data_string: Default::default(),
                datatype: None,
                smethod_combo_state: Self::create_smeth_combo(),
                sizing_method: None,
                sizing_meth_str: Default::default()
            }
        }
    pub fn is_valid_entry(&self, dat_str : &str) -> bool{
        if dat_str == ""{
            true
        }
        else if let Some(dat_type) = self.datatype{
            match dat_type{
                PacketDataType::U64 => dat_str.parse::<u64>().is_ok(),
                PacketDataType::U32 => dat_str.parse::<u32>().is_ok(),
                PacketDataType::U16 => dat_str.parse::<u16>().is_ok(),
                PacketDataType::U8  => dat_str.parse::<u8>().is_ok(),
                PacketDataType::I64 => dat_str.parse::<i64>().is_ok() || dat_str == "-",
                PacketDataType::I32 => dat_str.parse::<i32>().is_ok() || dat_str == "-",
                PacketDataType::I16 => dat_str.parse::<i16>().is_ok() || dat_str == "-",
                PacketDataType::I8  => dat_str.parse::<i8>().is_ok()  || dat_str == "-",
                _ => true
            }
        }
        else{
            true
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
                PacketDataType::CStr => std::ffi::CString::new(self.data_string.as_str()).unwrap().as_bytes_with_nul().iter().cloned().collect(),
                PacketDataType::U64 => self.data_string.parse::<u64>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::U32 => self.data_string.parse::<u32>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::U16 => self.data_string.parse::<u16>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::U8  => self.data_string.parse::<u8>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I64 => self.data_string.parse::<i64>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I32 => self.data_string.parse::<i32>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I16 => self.data_string.parse::<i16>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
                PacketDataType::I8  => self.data_string.parse::<i8>().expect("Invalid Integer Value").to_ne_bytes().iter().cloned().collect(),
            }
        }
        else{
            Default::default()
        }
    } 

    fn create_dtype_combo() -> ComboState<PacketDataType>{
        ComboState::new(
            vec![
                    PacketDataType::Bytes(SizingMethod::FixedSize(0)),
                    PacketDataType::CStr,
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
    fn create_smeth_combo() -> ComboState<SizingMethod>{
        ComboState::new(
            vec![
                SizingMethod::FixedSize(0),
                SizingMethod::SizeHeader(0)
            ]
        )
    }

    pub fn draw(&self, parent_index : usize) -> Element<'_, Message>{
        let mut row = Row::new();
        let idx = self.index.clone();
        row = row.push(text::Text::new(format!("{}", self.index)));
        row = row.push(
            ComboBox::new(
                &self.dtype_combo_state, "Please select a data type", self.datatype.as_ref(), 
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
            let field_idx = self.index.clone();
            let p_idx = parent_index.clone();
            row = row.push(
                combo_box(
                    &self.smethod_combo_state,
                     "Select a sizing Method",
                      self.sizing_method.as_ref(), 
                      move |x| Message::PVMessage(p_idx, PVMessage::ChangeSizingMethod(x, field_idx))
                    )
            ).width(Length::FillPortion(1));

            if let Some(meth) = self.sizing_method{
                let (f_idx, p_idx) = (self.index, parent_index);
                row = row.push(
                    match meth{
                        SizingMethod::SizeHeader(_) => text_input("Field index for Sizing", &self.sizing_meth_str)
                            .on_input(move |s| Message::PVMessage(p_idx, PVMessage::MethodEntry(s, f_idx))),
                        SizingMethod::FixedSize(_) => text_input("Size of Data", &self.sizing_meth_str)
                            .on_input(move |s| Message::PVMessage(p_idx, PVMessage::MethodEntry(s, f_idx))),
                    }
                );
            }
            
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