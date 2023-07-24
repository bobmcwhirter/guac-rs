use self::certify_vuln_m1::{PkgInputSpec, PackageQualifierInputSpec, VulnerabilityInput, VulnerabilityMetaDataInput, CVEInputSpec, OSVInputSpec, GHSAInputSpec};
use graphql_client::GraphQLQuery;
use packageurl::PackageUrl;
use std::str::FromStr;
use chrono::Utc;
use serde::{Deserialize, Serialize};

//#[derive(Debug, Serialize, Deserialize)]
type Time = chrono::DateTime<Utc>;

pub enum Vulnerability {
    Cve(Cve),
    Osv(Osv),
    Ghsa(Ghsa)
}

pub struct Cve {
    year: i64,
    cve_id: String,
}

pub struct Osv {
    osv_id: String,
}

pub struct Ghsa {
    ghsa_id: String,
}

pub struct Metadata {
    db_uri: String,
    db_version: String,
    scanner_uri: String,
    scanner_version: String,
    time_scanned: Time,
    origin: String,
    collector: String,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/certify_vuln.gql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CertifyVulnM1;

impl TryFrom<&str> for PkgInputSpec {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let purl = PackageUrl::from_str(s)?;
        let mut qualifiers = Vec::new();
        for (key, value) in purl.qualifiers().iter() {
            qualifiers.push(PackageQualifierInputSpec {
                key: key.to_string(),
                value: value.to_string(),
            })
        }

        Ok(PkgInputSpec {
            type_: purl.ty().to_string(),
            namespace: purl.namespace().map(|s| s.to_string()),
            name: purl.name().to_string(),
            subpath: purl.subpath().map(|s| s.to_string()),
            version: purl.version().map(|s| s.to_string()),
            qualifiers: if qualifiers.is_empty() {
                None
            } else {
                Some(qualifiers)
            },
        })
    }
}

impl TryFrom<Vulnerability> for VulnerabilityInput {
    type Error = anyhow::Error;

    fn try_from(vuln: Vulnerability) -> Result<Self, Self::Error> {
        match vuln {
            Vulnerability::Cve(cve) => {
                Ok(VulnerabilityInput {
                    cve: Some(CVEInputSpec {
                        cve_id: cve.cve_id,
                        year: cve.year
                    }),
                    osv: None,
                    ghsa: None,
                    no_vuln: None,
                })
            }
            Vulnerability::Osv(osv) => {
                Ok(VulnerabilityInput {
                    cve: None,
                    osv: Some( OSVInputSpec {
                        osv_id: osv.osv_id
                    }),
                    ghsa: None,
                    no_vuln: None,
                })
            }
            Vulnerability::Ghsa(ghsa) => {
                Ok(VulnerabilityInput {
                    cve: None,
                    osv: None,
                    ghsa: Some( GHSAInputSpec {
                        ghsa_id: ghsa.ghsa_id
                    } ),
                    no_vuln: None,
                })
            }
        }
    }
}

impl TryFrom<Metadata> for VulnerabilityMetaDataInput {
    type Error = anyhow::Error;

    fn try_from(meta: Metadata) -> Result<Self, Self::Error> {
        Ok(
            VulnerabilityMetaDataInput {
                db_uri: meta.db_uri,
                db_version: meta.db_version,
                scanner_uri: meta.scanner_uri,
                scanner_version: meta.scanner_version,
                time_scanned: meta.time_scanned,
                collector: meta.collector,
                origin: meta.origin,
            }
        )
    }
}


#[cfg(test)]
pub mod test {
    use crate::graphql::{GuacClient};
    use super::*;

    #[tokio::test]
    async fn ingest_vuln() -> Result<(), anyhow::Error> {

        let client = GuacClient::new("http://localhost:8080/query".into());

        let vuln = Vulnerability::Ghsa(
            Ghsa {
                ghsa_id: "ghsa-taco-vuln".to_string(),
            }
        );

        let meta = Metadata {
            db_uri: "http://db.example.com/".to_string(),
            db_version: "1.0".to_string(),
            scanner_uri: "collectorist-osv".to_string(),
            scanner_version: "1.0".to_string(),
            time_scanned: Default::default(),
            origin: "OSV".to_string(),
            collector: "collectorist-osv".to_string(),
        };

        let result = client.ingest_package(
            "pkg:maven/org.apache.logging.log4j/log4j-core@2.13.0",
        ).await?;

        let result = client.ingest_certify_vuln(
            "pkg:maven/org.apache.logging.log4j/log4j-core@2.13.0",
            vuln,
            meta,
        ).await?;

        //client.ingest_certify_vuln(

        //);

        Ok(())
    }
}