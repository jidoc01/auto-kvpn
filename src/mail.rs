use std::{collections::HashSet, net::TcpStream};

use imap::Session;
use native_tls::TlsStream;

pub struct MailHelper {
    session: Session<TlsStream<TcpStream>>
}

impl MailHelper {
    pub fn login(id: &str, password: &str) -> Option<MailHelper> {
        let tls = native_tls::TlsConnector::builder().build().unwrap();
        let client = imap::connect(
          ("imap.gmail.com", 993),
          "imap.gmail.com",
          &tls,
        ).unwrap();
        let session = client.login(id, password);
        match session {
            Ok(mut session) => {
                session.select("INBOX").unwrap();
                Some(MailHelper {
                    session: session
                })
            },
            Err(_err) => None
        }
    }

    pub fn search(&mut self, query: &str) -> HashSet<u32> {
        let session = &mut self.session;
        session.select("INBOX").unwrap();
        let uids = session.search(query).unwrap();
        uids
    }

    pub fn read_text(&mut self, uid: u32, query: &str) -> String {
        let session = &mut self.session;
        let messages = session.fetch(uid.to_string(), query).unwrap();
        let first = messages.first().unwrap();
        let bytes = first.text().unwrap().to_vec();
        let s = String::from_utf8(bytes).unwrap();
        s
    }
}

impl Drop for MailHelper {
    fn drop(&mut self) {
        self.session.logout().unwrap();
    }
}