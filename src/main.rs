use std::{ collections::HashMap, str::FromStr, thread::sleep, time::Duration };

use clap;
use reqwest;
use serde_json;
use lettre::Transport;

const MY_ADDR: &'static str = "sanyaogr1605@yandex.ru";
const MAIL_APP_PASS: &'static str = "atxqtilwcjdthado";

const RECIP_ADDR: &'static str = "sanya2003ogorodov@gmail.com";

fn parse_tickets_info<'a>(json_val: &'a serde_json::Value) -> Result<HashMap<&'a str, u64>, String> {

    let mut result = HashMap::new();

    let sets_object = json_val["sets"].as_object().ok_or("\'sets\' isn't an Object")?;
    for (set_key, set_val) in sets_object {

        let set_object = set_val.as_object().ok_or("\'set\' isn't an Object")?;
        
        let (_, ticket_type_val) = set_object
            .get_key_value("name")
            .ok_or(format!("{set_key} set doesn't have field \'name\'"))?;
        
        let (_, ticket_amount_val) = set_object
            .get_key_value("amount_vacant")
            .ok_or(format!("{set_key} set doesn't have field \'amount_vacant\'"))?;

        result.insert(
            ticket_type_val.as_str()
            .ok_or("failed to parse \'name\' to &str")?, 
            ticket_amount_val.as_u64()
            .ok_or("failed to parse \'amount_vacant\' to u64")?);
    }
    Ok(result)
}

fn recieve_tickets_info(client: &reqwest::blocking::Client, headers: reqwest::header::HeaderMap) -> Result<String, Box<dyn std::error::Error>> {

    // post request generated from the curl command
    let res = client.post("https://ticketscloud.com/v1/services/widget")
        .headers(headers)
        .body("{\"event\":\"64ef25548abcaf2fdfd48227\"}")
        .send()?
        .text()?;
    Ok(res)
}

fn send_email(my_address: &str, recip_address: &str, subject: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {

    let email_msg = lettre::Message::builder()
        .from(format!("Concert-Checker <{my_address}>").parse()?)
        .to(format!("Recipient <{recip_address}>").parse()?)
        .subject(subject)
        .body(message.to_owned())?;

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        "sanyaogr1605".to_owned(),
        MAIL_APP_PASS.to_owned());
    
    let mailer = lettre::SmtpTransport::relay("smtp.yandex.ru")?
        .port(465)
        .credentials(creds)
        .build();

    mailer.send(&email_msg)?;
    
    Ok(())
}

fn start_checking_loop(
    my_email: &str, 
    recip_email: &str, 
    client: &reqwest::blocking::Client, 
    headers: reqwest::header::HeaderMap, 
    predicate: Box<dyn Fn(&str, u64) -> bool>) -> Result<(), Box<dyn std::error::Error>> {
    
    loop {

        let res = match recieve_tickets_info(&client, headers.clone()) {

            Ok(response) => response,
            Err(err) => {

                send_email(my_email, recip_email, "Error", &format!("Error in recieve_tickets_info(): {err}"))?;
                continue;
            }
        };
        
        let json_res: serde_json::Value = serde_json::from_str(&res)?;
        
        match parse_tickets_info(&json_res) {

            Ok(info) => {
                
                let mut message = String::from_str("Watafaka chto za mazafaka")?;
                let mut vacant_exist = false;

                for (ticket_type, vacant_amount) in info {

                    if predicate(ticket_type, vacant_amount) == true {

                        vacant_exist = true;
                        message.push_str(
                            format!("\n\n{ticket_type} -> {vacant_amount}").as_str());
                    }
                }

                if vacant_exist {

                    send_email(my_email, recip_email, "O_o", &message)?;
                }
            },
            Err(err) =>  {

                send_email(my_email, recip_email, "Error", &format!("Error in parse_tickets_info(): {err}"))?;
            }
        };

        sleep(Duration::from_secs(30));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let matches = clap::Command::new("MyApp")
        .version("1.0")
        .about("Does awesome things")
        .arg(clap::arg!(--test)
            .required(false)
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let mut predicate: Box<dyn Fn(&str, u64) -> bool> = Box::new(|ticket_type: &str, vacant_amount: u64| {
        ticket_type != "Meet&Greet" && vacant_amount > 0
    });

    if matches.get_flag("test") == true {

        predicate = Box::new(|_, _| {
            true
        });
    }
    
    // request headers generated from the curl command
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("authority", "ticketscloud.com".parse().unwrap());
    headers.insert("accept", "*/*".parse().unwrap());
    headers.insert("accept-language", "ru,en;q=0.9".parse().unwrap());
    headers.insert("authorization", "token eyJhbGciOiJIUzI1NiIsImlzcyI6InRpY2tldHNjbG91ZC5ydSIsInR5cCI6IkpXVCJ9.eyJwIjoiNjRlYzkxNjMwMTI2MjQwY2VmYmE2ZTVkIn0.Jg_yvvFo5S2Ar4_cXjHBFrnmyLzSy5dEKn4VmqCAwv4".parse().unwrap());
    headers.insert("content-type", "application/json".parse().unwrap());
    //headers.insert(header::COOKIE, "tmr_lvid=ed55dfe4f6eaac2755b023b6cb6039d1; tmr_lvidTS=1714040943713; _gid=GA1.2.2090130421.1714040944; _ym_uid=1714040944666269717; _ym_d=1714040944; _ym_isad=1; domain_sid=77h4pQwrngVjytXwvyzzD%3A1714040944332; __stripe_mid=d70e20d4-c33d-4ad8-b75d-c93aeb228aa9e9edab; __stripe_sid=75e56784-294f-49c1-8eab-79496f69592caa7786; _ga=GA1.1.756654355.1714040944; tmr_detect=0%7C1714045308878; _ga_HKG8ET5SPT=GS1.1.1714044215.2.1.1714045476.60.0.0".parse().unwrap());
    headers.insert("origin", "https://ticketscloud.com".parse().unwrap());
    headers.insert("referer", "https://ticketscloud.com/v1/widgets/common?token=eyJhbGciOiJIUzI1NiIsImlzcyI6InRpY2tldHNjbG91ZC5ydSIsInR5cCI6IkpXVCJ9.eyJwIjoiNjRlYzkxNjMwMTI2MjQwY2VmYmE2ZTVkIn0.Jg_yvvFo5S2Ar4_cXjHBFrnmyLzSy5dEKn4VmqCAwv4&event=64ef25548abcaf2fdfd48227&s=1&r=1&org=64ec91630126240cefba6e5d&city=1512236&category=592841f8515e35002dead938&tags=592841f8515e35002dead939%2C%D0%9F%D0%BE%D0%BF%2C592841f8515e35002dead93c%2C%D0%AD%D0%BB%D0%B5%D0%BA%D1%82%D1%80%D0%BE%D0%BD%D0%B8%D0%BA%D0%B0%2C592841f8515e35002dead94a%2C%D0%A0%D1%8D%D0%BF%2F%D0%A5%D0%B8%D0%BF-%D1%85%D0%BE%D0%BF&lang=ru".parse().unwrap());
    //headers.insert("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"YaBrowser\";v=\"24.1\", \"Yowser\";v=\"2.5\"".parse().unwrap());
    //headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    //headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
    //headers.insert("sec-fetch-dest", "empty".parse().unwrap());
    //headers.insert("sec-fetch-mode", "cors".parse().unwrap());
    //headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
    //headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 YaBrowser/24.1.0.0 Safari/537.36".parse().unwrap());
    //headers.insert("x-requested-with", "XMLHttpRequest".parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    start_checking_loop(MY_ADDR, RECIP_ADDR, &client, headers, predicate)
}