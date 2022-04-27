use crate::errors::ClassifyError;
use cadence::{StatsdClient};
use maxminddb::{self, geoip2};
use std::{fmt, net::IpAddr, path::PathBuf, sync::Arc};

pub struct GeoIp {
    reader: Option<maxminddb::Reader<Vec<u8>>>,
    metrics: Arc<StatsdClient>,
}

pub struct ClientLocation<'a> {
    pub country: Option<&'a str>,
    pub region: Option<&'a str>,
}

impl GeoIp {
    pub fn builder() -> GeoIpBuilder {
        GeoIpBuilder::default()
    }

    pub fn locate(&self, ip: IpAddr) -> Result<ClientLocation, ClassifyError> {
        self.reader
            .as_ref()
            .ok_or_else(|| ClassifyError::new("No geoip database available"))?
            .lookup(ip)
            .map(|city_info: geoip2::City| ClientLocation {
                country: city_info.country.and_then(|c| c.iso_code),
                region: city_info
                    .subdivisions
                    .as_ref()
                    .and_then(|subs| subs.last())
                    .and_then(|sub| sub.iso_code),
            })
            .map_err(|err| err.into())
    }
}

impl Default for GeoIp {
    fn default() -> Self {
        GeoIp::builder().build().unwrap()
    }
}

// // maxminddb reader doesn't implement Debug, so we can't use #[derive(Debug)] on GeoIp.
impl fmt::Debug for GeoIp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "GeoIp {{ reader: {}, metrics: {:?} }}",
            if self.reader.is_some() {
                "Some(...)"
            } else {
                "None"
            },
            self.metrics
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct GeoIpBuilder {
    path: Option<PathBuf>,
    metrics: Option<Arc<StatsdClient>>,
}

impl GeoIpBuilder {
    pub fn path<P>(mut self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.path = Some(path.into());
        self
    }

    pub fn metrics(mut self, metrics: Arc<StatsdClient>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn build(self) -> Result<GeoIp, ClassifyError> {
        let reader = match self.path {
            Some(path) => Some(maxminddb::Reader::open_readfile(path)?),
            None => None,
        };
        let metrics = self.metrics.unwrap_or_else(|| {
            Arc::new(StatsdClient::from_sink("default", cadence::NopMetricSink))
        });
        Ok(GeoIp { reader, metrics })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_geoip_works() -> Result<(), Box<dyn std::error::Error>> {
        let geoip = super::GeoIp::builder()
            .path("./GeoIP2-City.mmdb")
            .build()?;

        // Test with an IP address in the UK to see whether the right subdivision is extracted.
        // This is the IP address of st-andrews.ac.uk, which should not change location anytime
        // soon.
        let ip = "138.251.7.84".parse()?;
        let location = geoip.locate(ip).unwrap();
        assert_eq!(location.country.unwrap(), "GB");
        assert_eq!(location.region.unwrap(), "FIF");
        Ok(())
    }
}
