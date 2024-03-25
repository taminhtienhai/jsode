// use crate::{core::{JsonToken, Punct}};

// pub struct JsonParser<'tk, Iter: Iterator<Item = char>> {
//     iter: TokenIter<'tk, Iter>,
// }

// pub enum JsonState {
//     Idle,
//     Object,
//     Array,
//     ObjProp,
//     ArrItem,
//     EOS, // End Of Stream
//     Error,
// }

// pub struct ParserStream<'tk, Iter: Iterator<Item = char>> {
//     iter: &'tk mut TokenIter<'tk, Iter>,
//     prev_token: Option<&'tk JsonToken>,
//     depth: usize,
//     state: JsonState,
// }



// impl <'tk, Iter: Iterator<Item = char>> JsonParser<'tk, Iter> {
//     pub fn parse(&mut self) {
//         while let Some(ref token) = self.iter.next() {
//             if let JsonToken::Punct(Punct::WhiteSpace, _) = token {
//                 continue;
//             }

//             if let JsonToken::Punct(Punct::OpenCurly, _) = token {
//                 continue;
//             }
//         }
//     }
// }

// impl <'tk, Iter: Iterator<Item = char>> ParserStream<'tk, Iter> {
//     pub fn change_state_to(&mut self, new_state: JsonState) {
//         self.state = new_state;
//     }
// }

// impl <'tk, Iter: Iterator<Item = char>> ParserStream<'tk, Iter> {
//     pub fn accept(&mut self) {

//     }

//     pub fn parse_obj_prop(&mut self) {
//         // poll until reaching ':'
//         loop {
//             let token = self.iter.next();

//             match token {
//                 Some(JsonToken::Punct(Punct::CloseCurly, _)) => break,
//                 None => break,
//                 _ => continue,
//             }
//         };
//     }

//     pub fn parse_array_item(&mut self) {
        
//     }

//     // pub fn parse_prop_value(&mut self) {
//     //     // poll until reaching ',' or '}'

//     //     self.change_state_to(JsonState::Idle);
//     // }

//     // pub fn parse_array_item(&mut self) {

//     //     self.change_state_to(JsonState::Idle);
//     // }
// }