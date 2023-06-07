use rusoto_s3::{S3, S3Client, GetObjectRequest, PutObjectRequest};
use std::io::Read;

use crate::errortype::ConnectorError;
use crate::models::log::Log;

const BUCKET: &'static str = "atlassian-connector";

pub async fn add_log(db: &S3Client, name: String, rec: Option<Vec<Log>>, id: String) -> Result<String, ConnectorError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&["event","status","attempt","time"]);
    if rec.is_some() {
        for re in rec.unwrap() {
            wtr.write_record(&[re.event, re.status, re.attempt.to_string(), re.time]);
        }
    }
    wtr.flush();
    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

    match db.put_object(PutObjectRequest {
        bucket: BUCKET.to_owned(),
        key: format!("{}/{}.csv", id, name),
        body: Some(data.into_bytes().into()),
        ..Default::default()
    }).await{
        Ok(_) => return Ok("Log successfuly created".to_owned()),
        Err(e) => return Err(ConnectorError::RusError(e.to_string()))
    }
}

pub async fn write_log(db: &S3Client, target_name: String, ev: String, stat: String, att: i32, tim: String, id: &str) {
    match get_one_log(db, target_name.clone(), id.to_string()).await {
        Ok(mut rec)=> {
            rec.push(Log { 
                event: ev,
                status: stat,
                attempt: att,
                time: tim,
            });
            add_log(db, target_name, Some(rec), id.to_string()).await;
        },
        Err(e) => println!("{:?}", e)
    }
}

pub async fn get_one_log(db: &S3Client, target_name: String, id: String) -> Result< Vec<Log>, ConnectorError>{
    match db.get_object(GetObjectRequest {
        bucket: BUCKET.to_owned(),
        key: format!("{}/{}.csv", id, target_name),
        ..Default::default()
    }).await {
        Ok(ob) =>{
            let result = tokio::task::spawn_blocking(|| {
                let mut object_data = ob.body.unwrap().into_blocking_read();
                let mut buffer = Vec::new();
                object_data.read_to_end(&mut buffer);

                let mut csv_reader = csv::ReaderBuilder::new()
                    .from_reader(std::io::BufReader::new(&buffer[..]));
                let records: Vec<Log> = csv_reader.deserialize::<Log>().map(|res| res.unwrap()).collect();
                return records
            }).await.expect("Task panicked");
            println!("{:?}", result);
            Ok(result)
        },
        Err(e) => 
        {
            return  Err(ConnectorError::RusError(e.to_string()))
        }    
    }
    
}