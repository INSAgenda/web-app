const CRASH_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head>
  <title>Crash report</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <script noinline src="https://js.sentry-cdn.com/5d343bdb601a48ef99cec222ee944943.min.js" crossorigin="anonymous"></script>
  <style>
    body {
      width: 50%;
      margin: 0 25%;
      display: flex;
      flex-direction: column;
      min-height: 100vh;
      justify-content: center;
    }
    @media screen and (max-width: 700px) {
      body {
        width: 96%;
        margin: 0 2%;
      }
    }
    
    h3 {
      font-weight: normal;
      font-size: 1rem;
      margin-bottom: .3rem;
    }
    
    pre {
      margin-top: .3rem;
      white-space: pre-wrap;
    }

    a {
      color: black;
      text-decoration: none;
      padding: .5rem;
      border: 1px solid #686AC4;
      border-radius: .25rem;
      transition: all .1s ease;
    }
    
    a:hover {
      box-shadow: 0px 5px 12px 0px rgba(0,0,0,0.4)
    }
    
    a[href^="mailto"] {
      background-color: #686AC4;
      color: white;
      margin-right: .5rem;
    }
    
    #title {
      display: flex;
      align-items: center;
    }
    
    #title>svg {
      fill: red;
    }
    
    h1 {
      font-size: 3rem;
      margin: 0 0 0 1rem;
      color: red;
    }
  </style>
</head>
<body>
  <div id="title">
    <svg xmlns="http://www.w3.org/2000/svg" height="4rem" viewBox="0 0 24 24" width="4rem" fill="\#000000"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/></svg>
    <h1>Fatal error</h1>
  </div>
  <p>Hmm, c'est gênant... La page a rencontré un problème inattendu. Pour éviter que cet incident se reproduise, veuillez nous envoyer un rapport.</p>
  <h3>Détails techniques:</h3>
  <pre>[MESSAGE]</pre>
  <br/>
  <div>
    <a href="mailto:support@insagenda.fr?subject=INSAgenda%20crash%20report&body=[ENCODED MESSAGE]">Rapporter</a>
    <a href="">Recharger la page</a>
  </div>
  <div style="height: 10rem;"></div>
  <script>Sentry.captureException("[MESSAGE]");</script>
</body>
</html>
"#;

use js_sys::{Reflect::get, Function};
use crate::prelude::*;

pub fn init() {
    std::panic::set_hook(Box::new(|info| {
        let window = window();
        let doc = window.doc().document_element().unwrap();

        let mut payload: Option<String> = info.payload().downcast_ref().map(|v: &String| v.to_owned());
        if payload.is_none() {
            if let Some(p2) = info.payload().downcast_ref::<&'static str>() {
                payload = Some(p2.to_string());
            }
        }

        let mut message = match (payload, info.location()) {
            (Some(payload), Some(location)) => format!("web-app panicked at '{}', {}", payload, location),
            (Some(payload), None) => format!("web-app panicked at '{}'", payload),
            (None, Some(location)) => format!("web-app panicked, {}", location),
            (None, None) => format!("web-app panicked, {:?}", info),
        };

        let encode_uri_component = get(&window, &"encodeURIComponent".into()).unwrap();
        let encode_uri_component: Function = encode_uri_component.dyn_into().unwrap();
        let encoded_message = encode_uri_component.call1(&window, &JsValue::from_str(&message)).unwrap();
        let encoded_message = encoded_message.as_string().unwrap();
        let html = CRASH_PAGE.replace("[ENCODED MESSAGE]", &encoded_message);

        message = message.replace('<', "&lt;");
        message = message.replace('>', "&gt;");

        let html = html.replace("[MESSAGE]", &message);
        doc.set_inner_html(&html);
    }));
}