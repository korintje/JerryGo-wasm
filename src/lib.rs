extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use goban;
extern crate console_error_panic_hook;

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


/*+++++++++++ Themes ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++*/
//#[derive(Debug, Serialize, Deserialize)]
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

/*
let themes_string = r#"{
						"Jerry": {"bancolor": "0E000E", "linecolor": "000000", "stone0_name": "jerry0.png", "stone1_name": "jerry1.png"}, 
						"Apple": {"bancolor": "0E00FF", "linecolor": "000000", "stone0_name": "apple0.png", "stone1_name": "apple1.png"},
					}"# ; 
let themes: BTreeMap<String, f64> = serde_json::from_str(s).unwrap();
*/
/*+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++*/

struct MyGame {
	game: goban::rules::game::Game,
	view: View,
	//theme: Themes,
}

struct View {
	canvas:				web_sys::HtmlCanvasElement,
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

	fn new(bansize: (u32, u32), canvas_id: &str, bansize_selector_id: &str, theme_selector_id: &str, forward_button_id: &str, backward_button_id: &str, reset_button_id: &str, export_button_id: &str) 
		-> MyGame {
					let document = web_sys::window().unwrap().document().unwrap() ;
					let mygame = MyGame {
											game:	goban::rules::game_builder::GameBuilder::default().size(bansize).rule(goban::rules::Rule::Japanese).build().unwrap(),
											view:	View {
														canvas:				document.get_element_by_id(canvas_id).unwrap().dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap(),
														bansize_selector:	document.get_element_by_id(bansize_selector_id).unwrap(),
														theme_selector:		document.get_element_by_id(theme_selector_id).unwrap(),
														forward_button:		document.get_element_by_id(forward_button_id).unwrap(),
														backward_button:	document.get_element_by_id(backward_button_id).unwrap(),
														reset_button:		document.get_element_by_id(reset_button_id).unwrap(),
														export_button:		document.get_element_by_id(export_button_id).unwrap(),
														coords:				size2grid_coords((9, 9), 463.0, 463.0),
														node:				size2grid_node((9, 9)),
														stars:				size2stars((9, 9)),
													},
											//theme:	Themes::Jerry,
										} ;
					log!("My Game has been generated") ; 
					log!("{:?}", mygame.view.canvas) ;
					mygame
				}

	fn draw_canvas (&self) {

		// Get canvas
		let ctx = self.view.canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap() ;

		// Draw gridlines of goban
		ctx.set_global_alpha(1.0);
		ctx.set_line_width(1.0);
		ctx.set_stroke_style(&JsValue::from("blue"));
		for (n, nlist) in self.view.node.iter().enumerate() {
			for (m, mlist) in nlist.iter().enumerate() {
				for (c, clist) in mlist.iter().enumerate() {
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
			ctx.arc(self.view.coords[star[0]][star[1]][0], self.view.coords[star[0]][star[1]][1], 3.0, 0.0, std::f64::consts::PI*2.0) ;
			ctx.fill();
		}

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
	console_error_panic_hook::set_once();
	log!("START") ; 
	//panic::set_hook(Box::new(console_error_panic_hook::hook));
	let bansize = (9,9) ;
	let mygame = MyGame::new(
		bansize,
		"canvas_0",
		"bansize_selector_0", 
		"theme_selector_0", 
		"forward_button_0", 
		"backward_button_0", 
		"reset_button_0", 
		"export_button_0",	
	) ;
	mygame.draw_canvas() ;

}