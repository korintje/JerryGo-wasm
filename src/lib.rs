extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use goban;
extern crate console_error_panic_hook;
use std::rc::Rc;
use std::cell::Cell;
use std::cell::RefCell;
use std::borrow::BorrowMut;

// Macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// Function to make a grid node of goban-eyes
fn size2grid_node (bansize: (u32, u32)) -> Vec<Vec<Vec<[usize; 2]>>> {
	
	let bansize_x = bansize.0 as usize ;
	let bansize_y = bansize.1 as usize ;

	let mut node = vec![vec![vec![]; bansize_x]; bansize_y] ;

	for (c, column) in node.iter_mut().enumerate() {
		for (r, row) in column.iter_mut().enumerate() {
			match c {
				0 => (), 
				_ => row.push([c-1, r]),
			}
			match r {
				0 => (),
				_ => row.push([c, r-1]),
			}
			if c != bansize_x - 1 {
				row.push([c+1, r]) ;
			}
			if r != bansize_y - 1 {
				row.push([c, r+1]) ;
			}
		}
	}

	log!("bansize = {}", bansize_x) ;
	log!("NODE: {:?}", node) ;
	node

}

// Function to make a grid coord of goban-eyes
fn size2grid_coords (bansize: (u32, u32), cv_w: f64, cv_h: f64 ) -> Vec<Vec<[f64; 2]>> {

	let ratio_edge2sep = 1.4 ; // 碁盤のマージン(両側の合計が線の間隔の何倍か)
	let margin_ban2cv = 17.0 ; // 座標表示用余白(px)。マージンに追加される値

	let sepx = (cv_w - margin_ban2cv) / (bansize.0 as f64 - 1.0 + ratio_edge2sep);
	let sepy = (cv_h - margin_ban2cv) / (bansize.1 as f64 - 1.0 + ratio_edge2sep); 
	let sep = vec![sepx, sepy].iter().fold(0.0/0.0, |m, v| v.min(m));
	//let sep = std::cmp::min(sepx, sepy)
	let margin_ban2line = sep * ratio_edge2sep / 2.0;

	let mut coords = vec![] ;
	for n in 0..bansize.0 {
		coords.push(vec![]) ;
		for m in 0..bansize.1 {
			let coord = [ (margin_ban2cv + margin_ban2line + sep * n as f64).round(), (margin_ban2cv + margin_ban2line + sep * m as f64).round() ] ;
			coords[n as usize].push(coord) ;
		}
	}

	log!("COORDS: {:?}", coords) ;
	coords 
}

// Function to make a grid coord of goban-stars
fn size2stars (bansize: (u32, u32)) -> Vec<Vec<usize>>	{
	let mut stars = vec![] ;
	let xlen = bansize.0 as usize;
	let ylen = bansize.1 as usize;
	if xlen % 2 == 1 && ylen % 2 == 1 {
		if xlen / 4 >= 3 && ylen / 4 >= 3 {
			let ixs = vec![3, (xlen / 2), (xlen - 4)] ;
			let iys = vec![3, (ylen / 2), (ylen - 4)] ;
			for &ix in ixs.iter() {
				for &iy in iys.iter() {
					let star = vec![ix, iy] ;
					stars.push(star) ;
				}
			}
		} else {
			let star = vec![(xlen / 2), (ylen / 2)] ;
			stars.push(star) ;
		}
	}

	log!("STARS: {:?}", stars) ;
	stars
}

//#[derive(Debug, Serialize, Deserialize)]
/*
enum Themes {
    Jerry {
        bancolor:  String,
        linecolor: String,
		linewidth: i32,
    },
    Apple {
        bancolor:  String,
        linecolor: String,
		linewidth: i32,
    },
}
*/

/*
let themes_string = r#"{
						"Jerry": {"bancolor": "0E000E", "linecolor": "000000", "stone0_name": "jerry0.png", "stone1_name": "jerry1.png"}, 
						"Apple": {"bancolor": "0E00FF", "linecolor": "000000", "stone0_name": "apple0.png", "stone1_name": "apple1.png"},
					}"# ; 
let themes: BTreeMap<String, f64> = serde_json::from_str(s).unwrap();
*/

struct MyGame {
	game: goban::rules::game::Game,
	view: View,
	//theme: Themes,
}

struct View {
	canvas_goban:		web_sys::HtmlCanvasElement,
	canvas_stone:		web_sys::HtmlCanvasElement,
	canvas_dummy:		web_sys::HtmlCanvasElement,
	bansize_selector:	web_sys::Element,
	theme_selector:		web_sys::Element,
	forward_button:		web_sys::Element,
	backward_button:	web_sys::Element,
	reset_button:		web_sys::Element,
	export_button:		web_sys::Element,
	coords:				Vec<Vec<[f64; 2]>>,
	stars:				Vec<Vec<usize>>,
	node:				Vec<Vec<Vec<[usize; 2]>>>,	
	}

// impl View {}

impl MyGame {

	fn new (bansize: (u32, u32), canvas_goban_id: &str, canvas_stone_id: &str, canvas_dummy_id: &str, bansize_selector_id: &str, theme_selector_id: &str, forward_button_id: &str, backward_button_id: &str, reset_button_id: &str, export_button_id: &str) 
		-> MyGame {
					let document = web_sys::window().unwrap().document().unwrap() ;
					let mygame = MyGame {
											game:	goban::rules::game_builder::GameBuilder::default().size(bansize).rule(goban::rules::Rule::Japanese).build().unwrap(),
											view:	View {
														canvas_goban:		document.get_element_by_id(canvas_goban_id).unwrap().dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap(),
														canvas_stone:		document.get_element_by_id(canvas_stone_id).unwrap().dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap(),
														canvas_dummy:		document.get_element_by_id(canvas_dummy_id).unwrap().dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap(),
														bansize_selector:	document.get_element_by_id(bansize_selector_id).unwrap(),
														theme_selector:		document.get_element_by_id(theme_selector_id).unwrap(),
														forward_button:		document.get_element_by_id(forward_button_id).unwrap(),
														backward_button:	document.get_element_by_id(backward_button_id).unwrap(),
														reset_button:		document.get_element_by_id(reset_button_id).unwrap(),
														export_button:		document.get_element_by_id(export_button_id).unwrap(),
														coords:				size2grid_coords(bansize, 463.0, 463.0),
														node:				size2grid_node(bansize),
														stars:				size2stars(bansize),
													},
											//theme:	Themes::Jerry,
										} ;
					log!("My Game has been generated") ; 
					log!("{:?}", mygame.view.canvas_goban) ;
					mygame
				}


	fn draw_ban (&self) {

		// Get canvas
		let ctx = self.view.canvas_goban.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap() ;

		// Draw gridlines of goban
		ctx.set_global_alpha(1.0);
		ctx.set_line_width(1.0);
		ctx.set_stroke_style(&JsValue::from("blue"));
		for (n, nlist) in self.view.node.iter().enumerate() {
			for (m, mlist) in nlist.iter().enumerate() {
				for clist in mlist.iter() {
					ctx.begin_path() ;
					ctx.move_to(self.view.coords[n as usize][m as usize][0], self.view.coords[n as usize][m as usize][1]);
					ctx.line_to(self.view.coords[clist[0] as usize][clist[1] as usize][0], self.view.coords[clist[0] as usize][clist[1] as usize][1]);
					ctx.close_path() ;
					ctx.stroke();
				}
			}
		}

		// Draw stars on goban
		ctx.set_fill_style(&JsValue::from("blue"));
		for star in &self.view.stars {
			ctx.begin_path() ;
			ctx.arc(self.view.coords[star[0]][star[1]][0], self.view.coords[star[0]][star[1]][1], 3.0, 0.0, std::f64::consts::PI*2.0).unwrap() ;
			ctx.fill();
		}

	}

	/*
	fn clear_layer (&self) {

	}
	*/


	fn put_stone (& mut self, cx: f64, cy: f64) {
		let stone_idx_r = self.get_index_radius(cx, cy) ;
		self.game.play(goban::rules::Move::Play(stone_idx_r.0 as u8, stone_idx_r.1 as u8)) ;
		let history = self.game.plays() ;
		let latest_goban = history.last().unwrap() ;
		self.clear_stone() ;
		for string in latest_goban.go_strings().iter() {
			match string {
				Some(i) => log!("String is {:?}, Color is {:?}, Coodination is {:?}", i.as_ref(), i.as_ref().color, i.as_ref().stones()),
				_ => (),
			}
			//log!("string: {:?}", string) ;
			//if string != None {
			//	self.draw_stone(string.stones().0, string.stones().1, stone_idx_r.2, 1.0) ;
			//}
		}
	}

	fn put_dummy (&self, cx: f64, cy: f64) {
		let stone_idx_r = self.get_index_radius(cx, cy) ;
		self.clear_dummy() ;
		self.draw_dummy(stone_idx_r.0, stone_idx_r.1, stone_idx_r.2, 0.6) ;
		log!("Dummy Index: ({}, {})", stone_idx_r.0, stone_idx_r.1) ;
	}

	fn get_index_radius (&self, cx: f64, cy: f64) -> (usize, usize, f64)  {

		// Get canvas
		// let ctx = self.view.canvas_dummy.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap() ;

		// Get size parameter of Goban
		let xidx ;
		let yidx ;
		let size = self.game.goban().size() ;
		let xlen = size.0 as usize;
		let ylen = size.1 as usize;
		let x0   = self.view.coords[0][0][0] ;
		let y0   = self.view.coords[0][0][1] ;
		let xl   = self.view.coords[xlen - 1][ylen - 1][0];
		let yl   = self.view.coords[xlen - 1][ylen - 1][1];
		let sepx = (xl - x0) / (xlen as f64 - 1.0) ;
		let sepy = (yl - y0) / (ylen as f64 - 1.0) ;

		// Get x index
		if cx <= x0 {
			xidx = 0 ;
		} else if cx > xl {
			xidx = xlen - 1 ;
		} else {
			xidx = ((cx - x0 + sepx / 2.0) / sepx) as usize ;
		}

		// Get y index
		if cy <= y0 {
			yidx = 0 ;
		} else if cy > yl {
			yidx = ylen - 1;
		} else {
			yidx = ((cy - y0 + sepy / 2.0) / sepy) as usize ;
		}

		// Calc stone radius
		let stone_lw = 1.0 ;
		let stone_radius = (sepx / 2.0 - stone_lw).round();

		// Return tuple
		(xidx, yidx, stone_radius)

	}

	fn clear_dummy (&self) {
		let ctx = self.view.canvas_dummy.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap() ;
		ctx.clear_rect(0.0, 0.0, 463.0, 463.0);
	}
	
	fn clear_stone (&self) {
		let ctx = self.view.canvas_stone.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap() ;
		ctx.clear_rect(0.0, 0.0, 463.0, 463.0);
	}

	fn draw_dummy (&self, xidx: usize, yidx: usize, stone_radius: f64, alpha: f64) {

		// Get canvas for dummy stone
		let ctx = self.view.canvas_dummy.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap() ;
		//ctx.clear_rect(0.0, 0.0, 463.0, 463.0);
		ctx.set_global_alpha(alpha);

		// Get turn
		let turn = self.game.turn() ;
		match turn {
			goban::rules::Player::Black => ctx.set_stroke_style(&JsValue::from("black")),
			goban::rules::Player::White => ctx.set_stroke_style(&JsValue::from("white")),
		} 

		// Draw dummy stone
		ctx.begin_path() ;
		ctx.arc(self.view.coords[xidx][yidx][0], self.view.coords[xidx][yidx][1], stone_radius, 0.0, std::f64::consts::PI*2.0).unwrap();
		ctx.fill();

	}

	fn draw_stone (&self, xidx: usize, yidx: usize, stone_radius: f64, alpha: f64) {

		// Get canvas for dummy stone
		let ctx = self.view.canvas_stone.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap() ;
		ctx.set_global_alpha(alpha);

		// Get turn
		let turn = self.game.turn() ;
		match turn {
			goban::rules::Player::Black => ctx.set_stroke_style(&JsValue::from("black")),
			goban::rules::Player::White => ctx.set_stroke_style(&JsValue::from("white")),
		} 

		// Draw dummy stone
		ctx.begin_path() ;
		ctx.arc(self.view.coords[xidx][yidx][0], self.view.coords[xidx][yidx][1], stone_radius, 0.0, std::f64::consts::PI*2.0).unwrap();
		ctx.fill();

	}

	//fn load_game (kifu: &str) {}

	//fn update_view (theme: Theme, game: goban::rules::game::Game) {}
	
	//fn move_forward (step: i32) {}

	//fn move_backward (step: i32) {}
	
	//fn move_to_initial () {}
	
	//fn export_view () -> &str {}

	//fn export_to_clipboard ()  {}

}

#[wasm_bindgen]
pub fn start () {

	// Enable console
	console_error_panic_hook::set_once();
	log!("START!!!") ; 

	// Create a Go-game instance
	let bansize = (9,9) ;
	let mygame = MyGame::new(
		bansize,
		"layer_goban",
		"layer_stone",
		"layer_dummy",
		"bansize_selector_0", 
		"theme_selector_0", 
		"forward_button_0", 
		"backward_button_0", 
		"reset_button_0", 
		"export_button_0",	
	) ;
	let mygame_rc = Rc::new(RefCell::new(mygame)) ;

	// Draw Goban for the Go-game
	// mygame.draw_ban() ;
	mygame_rc.as_ref().borrow().draw_ban() ;

	// Bind Events to the Goban
	
	let entered = Rc::new(Cell::new(false)) ;

	{
		//let mygame_closure = Rc::clone(&mygame_rc) ;
		let entered = entered.clone() ;
		let closure = Closure::wrap(Box::new(move|_event: web_sys::MouseEvent|{
			entered.set(true);
		})as Box<dyn FnMut(_)>) ;
		mygame_rc.as_ref().borrow().view.canvas_stone.add_event_listener_with_callback("mouseenter", closure.as_ref().unchecked_ref()).unwrap() ;
		closure.forget() ;
	}

	{
		let mygame_closure = mygame_rc.clone() ;
		let entered = entered.clone() ;
		let closure = Closure::wrap(Box::new(move|event: web_sys::MouseEvent|{
			if entered.get() {
				mygame_closure.as_ref().borrow_mut().put_dummy(event.offset_x() as f64, event.offset_y() as f64) ;
			}
		})as Box<dyn FnMut(_)>) ;
		mygame_rc.as_ref().borrow().view.canvas_stone.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).unwrap() ;
		closure.forget() ;
	}

	{
		let mygame_closure = mygame_rc.clone() ;
		let entered = entered.clone() ;
		let closure = Closure::wrap(Box::new(move|_event: web_sys::MouseEvent|{
			mygame_closure.as_ref().borrow_mut().clear_dummy() ;
			entered.set(false) ;
		})as Box<dyn FnMut(_)>) ;
		mygame_rc.as_ref().borrow().view.canvas_stone.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref()).unwrap() ;
		closure.forget() ;
	}

	{
		let mygame_closure = mygame_rc.clone() ;
		let entered = entered.clone() ;
		let closure = Closure::wrap(Box::new(move|event: web_sys::MouseEvent|{
			if entered.get() {
				mygame_closure.as_ref().borrow_mut().put_stone(event.offset_x() as f64, event.offset_y() as f64) ;
			}
		})as Box<dyn FnMut(_)>) ;
		mygame_rc.as_ref().borrow_mut().view.canvas_stone.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref()).unwrap() ;
		closure.forget() ;
	}
	
}