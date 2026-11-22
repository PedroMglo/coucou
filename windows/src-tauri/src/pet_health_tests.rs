use super::*;
use std::cell::Cell;
fn bytes(source:Source)->Result<Vec<u8>,Failure>{Ok(match source{Source::Memory=>b"MemTotal: 1024 kB\nMemAvailable: 10 kB\n".to_vec(),Source::Cpu=>b"cpu 1 2 3 4 5 6 7 8 90 100\n".to_vec(),Source::Uptime=>b"100.5 42.0\n".to_vec()})}
#[test]fn guardian_health_final_publish_rechecks_authority_after_worker_completion(){
 let allowed=Cell::new(true);let sample=observe("agent_system_guardian","original",||allowed.get(),bytes).unwrap();
 allowed.set(false);assert!(publish(sample,||allowed.get()).is_err());
 let sample=observe("agent_system_guardian","original",||true,bytes).unwrap();assert!(publish(sample,||true).is_ok());
}
#[test]fn guardian_health_finding_has_evidence_without_model_or_guest_double_count(){
 let value=observe("agent_system_guardian",&"a".repeat(32),||true,bytes).unwrap();
 assert_eq!(value.memory.value.as_ref().unwrap().total_bytes,1048576);assert_eq!(value.cpu.value.as_ref().unwrap().total_ticks,36);assert_eq!(value.cpu.value.as_ref().unwrap().idle_ticks,9);
 assert_eq!(value.findings.len(),1);let finding=&value.findings[0];assert_eq!(finding.id,"health.memory.available.low");assert_eq!(finding.confidence,"observed");assert_eq!(finding.available_bytes,10240);assert_eq!(finding.status,"sampleOnly");
 assert!(!value.inference_used);let data=serde_json::to_vec(&value).unwrap();assert!(data.len()<=value.max_result_bytes);assert!(value.limitations.contains(&"cpuTicksNeedSeparateBaselineNoUsagePercent"));
}
#[test]fn guardian_health_partial_failure_does_not_become_healthy_or_erase_other_metrics(){
 let value=observe("agent_system_guardian","revision",||true,|s|if matches!(s,Source::Memory){Err(Failure::Unavailable)}else{bytes(s)}).unwrap();
 assert!(!value.memory.available&&value.cpu.available&&value.uptime.available);assert_eq!(value.memory.error,Some(Failure::Unavailable));assert!(value.findings.is_empty());
}
#[test]fn guardian_health_revocation_during_read_discards_sample_and_stops_further_io(){
 let valid=Cell::new(true);let reads=Cell::new(0);
 let answer=observe("agent_system_guardian","revision",||valid.get(),|s|{reads.set(reads.get()+1);valid.set(false);bytes(s)});
 assert!(answer.is_err());assert_eq!(reads.get(),1);
 let reads=Cell::new(0);assert!(observe("agent_system_guardian","revision",||false,|s|{reads.set(reads.get()+1);bytes(s)}).is_err());assert_eq!(reads.get(),0);
}
#[test]fn guardian_health_malformed_metrics_are_not_silently_coerced_or_invented(){
 for text in ["MemTotal: 1 kB\nMemAvailable: 2 kB\n","MemTotal: 1 kB\nMemTotal: 2 kB\nMemAvailable: 0 kB\n","MemTotal: 18446744073709551615 kB\nMemAvailable: 0 kB\n","MemTotal: 1 bytes\nMemAvailable: 0 kB\n","MemTotal: 1 kB\n"]{assert!(memory(text.as_bytes()).is_err());}
 for text in ["cpu0 1 2 3 4","cpu 1 2 3","cpu 18446744073709551615 1 1 1"]{assert!(cpu(text.as_bytes()).is_err());}
 for text in ["NaN 1","inf 1","-1 1","1 1 injected"]{assert!(uptime(text.as_bytes()).is_err());}
 assert!(cpu(&[255]).is_err());assert_eq!(memory(&vec![b'x';8193]).err(),Some(Failure::LimitExceeded));
}
#[cfg(target_os="linux")]
#[test]fn guardian_health_reader_bounds_before_materialization_and_refuses_symlinks(){
 use std::os::unix::fs::{DirBuilderExt,OpenOptionsExt,symlink};
 use std::io::Write;
 let root=std::env::temp_dir().join(format!("guardian-health-{}-{}",std::process::id(),now()));std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
 let path=root.join("metric");let mut file=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).unwrap();file.write_all(b"123456789\nprivate extra data").unwrap();drop(file);
 assert_eq!(bounded_file(&path,8,false).err(),Some(Failure::LimitExceeded));assert_eq!(bounded_file(&path,10,true).unwrap(),b"123456789\n");
 let link=root.join("link");symlink(&path,&link).unwrap();assert_eq!(bounded_file(&link,100,false).err(),Some(Failure::Unavailable));assert!(bounded_file(&root,100,false).is_err());std::fs::remove_dir_all(root).unwrap();
}
