use std::{cmp::Ordering, time::Instant};

use pollster::FutureExt as _;
use winit::{
    event::{Event, WindowEvent, KeyEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    keyboard::{Key, NamedKey}
};

use crate::graphics::State;  

/*
!
! I HAVE DECIDED TO ABANDON THIS PROJECT DUE TO ME NOT BEING ARSED TO DEAL WITH GLUTIN AND OPENGL SHIT JUST YET
!
! update from 15 months later: IM BACK BABY!
*/

mod graphics;

fn main(){
    run().block_on();
}

async fn run() {
    let mut timer = Instant::now();
    let event_loop = EventLoop::new().unwrap();
    
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    
    //let win_buil = glutin::window::WindowBuilder::new()
    //.with_title("test")
    //.with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    //let windowed_contest = glutin::ContextBuilder::new().build_windowed(win_buil, &evnt_lp).unwrap();
    
        
    let mut state = State::new(window).await;

    event_loop.run(move |event, cockandball|{
        cockandball.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { window_id, event } if window_id == state.window().id()=> if !state.input(&event) {
                match event {
                    WindowEvent::Resized(new_size) => state.resize(new_size),
                    WindowEvent::CloseRequested |
                    WindowEvent::KeyboardInput { event: 
                        KeyEvent{ logical_key: Key::Named(NamedKey::Escape), state: ElementState::Pressed, ..},
                        .. 
                    } => cockandball.exit(),
                    WindowEvent::MouseInput { .. } => {
                        state.input(&event);
                        state.window().request_redraw();
                    },
                    //WindowEvent::MouseInput { device_id, state, button } => state.window().set_window_level(window::WindowLevel::AlwaysOnTop),
                    WindowEvent::RedrawRequested => {
                        let fps = 1.0/timer.elapsed().as_secs_f32();
                        dbg!(fps);
                        timer = Instant::now();
                        match state.render() {
                            Ok(_) => {},
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => cockandball.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }

                    },
                    
                    _ => ()
                }
            },
            Event::UserEvent(_) => todo!(),
            Event::AboutToWait => state.window().request_redraw(),
            _ => ()
        }
       
    }).unwrap();
}

#[test]
fn comp_shader(){
    
}