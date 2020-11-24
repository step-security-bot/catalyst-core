use crate::jcli_app::rest::Error;
use crate::jcli_app::utils::{DebugFlag, HostAddr, OutputFormat, RestApiSender, TlsCert};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Stats {
    /// Get network information
    Get {
        #[structopt(flatten)]
        addr: HostAddr,
        #[structopt(flatten)]
        debug: DebugFlag,
        #[structopt(flatten)]
        output_format: OutputFormat,
        #[structopt(flatten)]
        tls: TlsCert,
    },
}

impl Stats {
    pub fn exec(self) -> Result<(), Error> {
        let Stats::Get {
            addr,
            debug,
            output_format,
            tls,
        } = self;
        let url = addr.with_segments(&["v0", "network", "stats"])?.into_url();
        let builder = reqwest::blocking::Client::new().get(url);
        let response = RestApiSender::new(builder, &debug, &tls).send()?;
        response.ok_response()?;
        let status = response.body().json_value()?;
        let formatted = output_format.format_json(status)?;
        println!("{}", formatted);
        Ok(())
    }
}