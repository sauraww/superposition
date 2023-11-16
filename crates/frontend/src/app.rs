use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub contexts: Vec<Context>,
    pub overrides: Map<String, Value>,
    pub default_configs: Map<String, Value>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Context {
    pub id: String,
    pub condition: Value,
    pub override_with_keys: [String; 1],
}

pub async fn fetch_config(tenant: String) -> Result<Config, String> {
    let client = reqwest::Client::new();
    let host = match std::env::var("APP_ENV").as_deref() {
        Ok("PROD") => {
            "https://context-aware-config.sso.internal.svc.k8s.apoc.mum.juspay.net"
        }
        Ok("SANDBOX") => "https://context-aware.internal.staging.mum.juspay.net",
        _ => "http://localhost:8080",
    };
    let url = format!("{host}/config");
    match client.get(url).header("x-tenant", tenant).send().await {
        Ok(response) => {
            let config: Config = response.json().await.map_err(|e| e.to_string())?;
            Ok(config)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);
    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/tailwind.css"/>
        <Link href="https://cdn.jsdelivr.net/npm/bootstrap@5.0.2/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-EVSTQN3/azprG1Anm3QDgpJLIm9Nao0Yz1ztcQTwFspd3yD65VohhpuuCOmLASjC" crossorigin="anonymous"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        // sets the document title
        <Title text="Welcome to Context Aware Config"/>
        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route ssr=SsrMode::PartiallyBlocked path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let query = use_query_map(cx);

    let tenant =
        query.with(|params_map| params_map.get("tenant").cloned().unwrap_or_default());
    let config_data =
        create_blocking_resource(cx, || {}, move |_| fetch_config(tenant.clone()));

    view! { cx,
        <div class="container mt-5" >
        <div class="text-center mb-4">
        <h3  class="fw-bold">"Welcome to Context Aware Config!"</h3>
         </div>
         <Suspense fallback=move || view! {cx, <p>"Loading (Suspense Fallback)..."</p> }>
         {
             config_data.with(cx, move |result| {
                 match result {
                     Ok(config) => {
                         let rows = |k:&String, v:&Value| {
                             let key = k.replace("\"", "").trim().to_string();
                              let value = format!("{}", v).replace("\"", "").trim().to_string();
                              view! { cx,
                                  <tr>
                                      <td class="fw-normal col w-50 shadow-sm"> <div class ="col"> {key}</div></td>
                                      <td class="fw-normal col w-50 shadow-sm"><div class ="col">{value}</div></td>
                                  </tr>
                              }
                         };

                         let contexts_views: Vec<_> = config.contexts.iter().map(|context| {
                             let condition = extract_and_format(&context.condition);
                             let rows: Vec<_> = context.override_with_keys.iter()
                             .filter_map(|key| config.overrides.get(key))
                             .flat_map(|ovr| ovr.as_object().unwrap().iter())
                             .map(|(k, v)| {
                                 rows(&k,&v)
                             }).collect();

                             view! { cx,
                                 <h6 class="fw-normal font-monospace">"Condition: " <span class="badge rounded-pill bg-secondary small">  {&condition} </span> </h6>
                                 <table class="table table-responsive table-bordered table-hover border-secondary">
                                 <thead class="table-primary border-secondary">
                                 <tr>
                                   <th>Key</th>
                                   <th>Value</th>
                                 </tr>
                             </thead>
                                 <tbody class="bg-light">
                                 { rows }
                                 </tbody>
                                 </table>
                             }
                        }).collect::<Vec<_>>();

                         let new_context_views = contexts_views.into_iter().rev().collect::<Vec<_>>();
                         let default_config: Vec<_> = config.default_configs.iter().map(|(k,v)|{
                             rows(&k,&v)
                         }).collect();

                         vec![
                             view! { cx,
                                 <div class="mb-4 ">
                                 { new_context_views }
                                     <h6 class="mb-3 f-6 fw-normal font-monospace">"Default Configuration"</h6>
                                     <table class="table table-responsive table-striped table-bordered table-hover border-secondary ">
                                     <thead class="table-primary border-secondary">
                                     <tr>
                                       <th>Key</th>
                                       <th>Value</th>
                                     </tr>
                                 </thead>
                                         <tbody>
                                             {default_config}
                                         </tbody>
                                     </table>
                                 </div>
                             }
                         ]
                     },
                     Err(error) => {
                         vec![
                             view! { cx,
                                 <div class="error">
                                     {"Failed to fetch config data: "}
                                     {error}
                                 </div>
                             }
                         ]
                     }
                 }
             })
         }
         </Suspense>
         </div>


    }
}
/// 404 - Not Found
#[component]
fn NotFound(cx: Scope) -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>(cx);
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }
    view! { cx,
        <h1>"Not Found"</h1>
    }
}

pub fn extract_and_format(condition: &Value) -> String {
    if condition.is_object() && condition.get("and").is_some() {
        // Handling complex "and" conditions
        let empty_vec = vec![];
        let conditions_json = condition
            .get("and")
            .and_then(|val| val.as_array())
            .unwrap_or(&empty_vec); // Default to an empty vector if not an array

        let mut formatted_conditions = Vec::new();
        for cond in conditions_json {
            formatted_conditions.push(format_condition(cond));
        }

        formatted_conditions.join(" and ")
    } else {
        // Handling single conditions
        format_condition(condition)
    }
}

fn format_condition(condition: &Value) -> String {
    if let Some(ref operator) = condition.as_object().and_then(|obj| obj.keys().next()) {
        let empty_vec = vec![];
        let operands = condition[operator].as_array().unwrap_or(&empty_vec);

        // Handling the "in" operator differently
        if operator.as_str() == "in" {
            let left_operand = &operands[0];
            let right_operand = &operands[1];

            let left_str = if left_operand.is_string() {
                format!("\"{}\"", left_operand.as_str().unwrap())
            } else {
                format!("{}", left_operand)
            };

            if right_operand.is_object() && right_operand["var"].is_string() {
                let var_str = right_operand["var"].as_str().unwrap();
                return format!("{} {} {}", left_str, operator, var_str);
            }
        }

        // Handling regular operators
        if let Some(first_operand) = operands.get(0) {
            if first_operand.is_object() && first_operand["var"].is_string() {
                let key = first_operand["var"].as_str().unwrap_or("UnknownVar");
                if let Some(value) = operands.get(1) {
                    if value.is_string() {
                        return format!(
                            "{} {} \"{}\"",
                            key,
                            operator,
                            value.as_str().unwrap()
                        );
                    } else {
                        return format!("{} {} {}", key, operator, value);
                    }
                }
            }
        }
    }

    "Invalid Condition".to_string()
}