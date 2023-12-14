use std::io::StdoutLock;

use serde::{Serialize, Deserialize};
use anyhow::{Context, Ok, bail};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
   Echo {echo: String},
   EchoOk {echo: String},
}

struct EchoNode {
    id: usize,
}

impl EchoNode {
    pub fn step (
        &mut self,
        input: Message,
        output: &mut serde_json::Serializer<StdoutLock>,
    ) -> anyhow::Result<()> {

        match input.body.payload {
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.src,
                    dst: input.dst,
                    body: Body { 
                        id: Some(self.id), 
                        in_reply_to: input.body.id, 
                        payload: Payload::EchoOk { echo}
                    },
                };
                reply
                    .serialize(output)
                    .context("serialize response to echo")?;
            },
            Payload::EchoOk { echo } => {},
        }

       
        self.id += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()>{
   
   let stdin = std::io::stdin().lock();
   let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();
   
   let stdout = std::io::stdout().lock();
   let mut output = serde_json::Serializer::new(stdout);

   let mut state = EchoNode {id: 0};
   
   for input in inputs {
    let input = input.context("Mealstrom input from STDIN could not be deserialized")?;
    state
        .step(input, &mut output)
        .context("Node step function failed")?;

   }

   Ok(())
}
