use iced::Element;



mod packet;
mod state;

fn update(state : &mut state::State, msg : state::Message){
    state.update(msg);
}

fn view(state : &crate::state::State) -> Element<state::Message>{
    state.draw()
}

fn main() -> iced::Result{
    iced::run("Test", update, view)
}


