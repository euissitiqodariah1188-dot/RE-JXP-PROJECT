// jpxhash.rs - JpXhash Custom Encoding Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

fn encode_map() -> HashMap<char, &'static str> {
    let mut m = HashMap::new();
    m.insert('A', "^$HBIE247842BC9U"); m.insert('B', "4&^UVB47FCB92UV");
    m.insert('C', "9*KJH78DFG32HJ5"); m.insert('D', "2@MNB65RTY89UI7");
    m.insert('E', "7#QWE12ASD45DF6"); m.insert('F', "5$RFV89TGB23YH8");
    m.insert('G', "1%UJM56KIO12KL9"); m.insert('H', "8^YHN34BGT78OP0");
    m.insert('I', "3&EDC90PLK56HY4"); m.insert('J', "6*IKM78NBV21ED3");
    m.insert('K', "4@QAZ12WSX34CDE"); m.insert('L', "9#RFV56YHN78UJM");
    m.insert('M', "2$TGB89PLK01QAZ"); m.insert('N', "7%UJM23EDC45RFV");
    m.insert('O', "1^YHN67BGT89TGB"); m.insert('P', "5&EDC90IKM12PLK");
    m.insert('Q', "3*RFV34QAZ56WSX"); m.insert('R', "8@TGB78UJM90EDC");
    m.insert('S', "4#YHN12PLK23RFV"); m.insert('T', "9$EDC45QAZ67TGB");
    m.insert('U', "2%RFV78UJM89YHN"); m.insert('V', "7^TGB90PLK01IKM");
    m.insert('W', "1&UJM23EDC45RFV"); m.insert('X', "5*YHN56TGB78UJM");
    m.insert('Y', "3@PLK89QAZ12WSX"); m.insert('Z', "8#RFV34EDC56TGB");
    m.insert('0', "!@#ZERO123!@#"); m.insert('1', "$%^ONE456$%^");
    m.insert('2', "&*TWO789&*"); m.insert('3', "()THREE012()");
    m.insert('4', "-+FOUR345-+"); m.insert('5', "[]FIVE678[]");
    m.insert('6', "{}SIX901{}"); m.insert('7', ";:SEVEN234;:");
    m.insert('8', "\",EIGHT567\","); m.insert('9', "<>NINE890<>");
    m.insert(' ', "___SPACE___"); m.insert('.', "...DOT...");
    m.insert(',', ",,,COMMA,,,"); m.insert('!', "!!!EXCL!!!");
    m.insert('?', "???QUES???"); m.insert('@', "@@@AT@@@");
    m.insert('#', "###HASH###"); m.insert('$', "$$$DOL$$$");
    m.insert('%', "%%%PER%%%"); m.insert('&', "&&&AMP&&&");
    m.insert('*', "***STR***"); m.insert('-', "---DASH---");
    m.insert('_', "___UND___"); m.insert('+', "+++PLS+++");
    m.insert('=', "===EQL==="); m.insert('/', "///FSL///");
    m.insert(':', ":::COL:::"); m.insert(';', ";;;SCL;;;");
    m
}

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("jpxhash_encode", JpxValue::NativeFunction(Arc::new(|args| {
            let text = args.first().map(|v| v.to_string().to_uppercase()).unwrap_or_default();
            let map = encode_map();
            let encoded: String = text.chars()
                .map(|c| map.get(&c).map(|s| s.to_string()).unwrap_or_else(|| c.to_string()))
                .collect();
            Ok(JpxValue::String(encoded))
        }))),
        ("jpxhash_decode", JpxValue::NativeFunction(Arc::new(|args| {
            let text = args.first().map(|v| v.to_string()).unwrap_or_default();
            let map = encode_map();
            // Build reverse map
            let mut reverse: Vec<(String, char)> = map.iter()
                .map(|(c, s)| (s.to_string(), *c))
                .collect();
            reverse.sort_by(|a, b| b.0.len().cmp(&a.0.len())); // longest first

            let mut result = String::new();
            let mut i = 0;
            let chars: Vec<char> = text.chars().collect();
            while i < chars.len() {
                let rest: String = chars[i..].iter().collect();
                let mut found = false;
                for (pattern, ch) in &reverse {
                    if rest.starts_with(pattern.as_str()) {
                        result.push(*ch);
                        i += pattern.chars().count();
                        found = true;
                        break;
                    }
                }
                if !found {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            Ok(JpxValue::String(result))
        }))),
        ("jpxhash_encode_letter", JpxValue::NativeFunction(Arc::new(|args| {
            let ch = args.first().map(|v| v.to_string().to_uppercase()).unwrap_or_default();
            let map = encode_map();
            let encoded = ch.chars().next()
                .and_then(|c| map.get(&c))
                .map(|s| s.to_string())
                .unwrap_or(ch);
            Ok(JpxValue::String(encoded))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    // Register JpXhash object
    let mut jpxhash_obj = HashMap::new();
    jpxhash_obj.insert("encode".to_string(), fns[0].1.clone());
    jpxhash_obj.insert("decode".to_string(), fns[1].1.clone());
    jpxhash_obj.insert("encode_letter".to_string(), fns[2].1.clone());
    interp.env.insert("JpXhash".to_string(), JpxValue::Object(jpxhash_obj.clone()));
    interp.builtins.insert("JpXhash".to_string(), JpxValue::Object(jpxhash_obj));
    Ok(())
}
