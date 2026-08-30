use sqlx::{sqlite::SqlitePoolOptions,SqlitePool};
async fn apply(pool:&SqlitePool,sql:&'static str){sqlx::raw_sql(sql).execute(pool).await.unwrap();}
#[tokio::test]
async fn stage2_database_upgrades_to_audit_and_corrections_without_data_loss(){
 let p=SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
 for migration in[
  include_str!("../../migrations/0001_bootstrap.sql"),include_str!("../../migrations/0002_core_domain.sql"),
  include_str!("../../migrations/0003_inspections.sql"),include_str!("../../migrations/0004_colony_events.sql"),
  include_str!("../../migrations/0005_colony_divisions.sql"),include_str!("../../migrations/0006_feedings.sql"),
  include_str!("../../migrations/0007_production.sql"),include_str!("../../migrations/0008_colony_movements.sql"),
  include_str!("../../migrations/0009_box_maintenance.sql"),include_str!("../../migrations/0010_colony_lifecycle.sql"),
  include_str!("../../migrations/0011_movement_documents.sql"),include_str!("../../migrations/0012_inspection_photos.sql"),
  include_str!("../../migrations/0013_box_state_integrity.sql"),]{apply(&p,migration).await;}
 sqlx::query("INSERT INTO meliponaries(id,name)VALUES('m1','Principal')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO species(id,common_name)VALUES('s1','Jataí')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO boxes(id,meliponary_id,code)VALUES('b1','m1','CX-001')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO colonies(id,meliponary_id,species_id,code,installed_at)VALUES('c1','m1','s1','JAT-001','2026-01-01 09:00:00')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO colony_box_occupancies(id,colony_id,box_id,started_at,ended_at)VALUES('o1','c1','b1','2026-01-01 09:00:00','2026-06-01 09:00:00')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO inspections(id,colony_id,box_id,inspected_at,strength)VALUES('i1','c1','b1','2026-01-10 10:00:00','medium')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO inspection_photos(id,inspection_id,relative_path,original_name,mime_type,byte_size,captured_at)VALUES('ph1','i1','media/inspections/i1/ph1.jpg','ph1.jpg','image/jpeg',42,'2026-01-10 10:01:00')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO feedings(id,colony_id,box_id,fed_at,food_type,quantity,unit)VALUES('f1','c1','b1','2026-01-15 10:00:00','Xarope',10,'ml')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO production_records(id,colony_id,box_id,harvested_at,product_type,quantity,unit)VALUES('p1','c1','b1','2026-02-01 10:00:00','honey',20,'ml')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO box_maintenance_records(id,box_id,colony_id,maintained_at,maintenance_type)VALUES('bm1','b1','c1','2026-02-10 10:00:00','repair')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO colony_events(id,colony_id,box_id,event_type,occurred_at,severity)VALUES('e1','c1','b1','observation','2026-02-15 10:00:00','info')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO colony_movements(id,colony_id,movement_type,moved_at,from_meliponary_id,from_box_id,destination)VALUES('mv1','c1','transport','2026-03-01 10:00:00','m1','b1','Feira')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO movement_documents(id,movement_id,document_type,reference_number)VALUES('d1','mv1','other','REF-1')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO colony_lifecycle_records(id,colony_id,box_id,action,occurred_at,previous_status,new_status,reason)VALUES('lc1','c1','b1','deactivate','2026-06-01 09:00:00','active','inactive','Pausa')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO colonies(id,meliponary_id,species_id,code,origin_type,installed_at,mother_colony_id)VALUES('c2','m1','s1','JAT-002','multiplication','2026-04-01 10:00:00','c1')").execute(&p).await.unwrap();
 sqlx::query("INSERT INTO colony_divisions(id,parent_colony_id,daughter_colony_id,source_box_id,performed_at,result)VALUES('dv1','c1','c2','b1','2026-04-01 10:00:00','successful')").execute(&p).await.unwrap();
 apply(&p,include_str!("../../migrations/0014_audit_and_record_corrections.sql")).await;
 for(table,expected)in[("meliponaries",1_i64),("species",1),("boxes",1),("colonies",2),("colony_box_occupancies",1),("inspections",1),("inspection_photos",1),("feedings",1),("production_records",1),("box_maintenance_records",1),("colony_events",1),("colony_movements",1),("movement_documents",1),("colony_lifecycle_records",1),("colony_divisions",1)]{let q=format!("SELECT COUNT(*) FROM {table}");let count:i64=sqlx::query_scalar(&q).fetch_one(&p).await.unwrap();assert_eq!(count,expected,"table {table}");}
 let audit_table:i64=sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_records'").fetch_one(&p).await.unwrap();assert_eq!(audit_table,1);
 let columns:Vec<String>=sqlx::query_scalar("SELECT name FROM pragma_table_info('inspections')").fetch_all(&p).await.unwrap();assert!(columns.contains(&"voided_at".to_owned()));assert!(columns.contains(&"corrected_at".to_owned()));
 let relation:String=sqlx::query_scalar("SELECT c.code FROM inspection_photos p JOIN inspections i ON i.id=p.inspection_id JOIN colonies c ON c.id=i.colony_id WHERE p.id='ph1'").fetch_one(&p).await.unwrap();assert_eq!(relation,"JAT-001");
 let legacy:String=sqlx::query_scalar("SELECT common_name FROM species WHERE id='s1'").fetch_one(&p).await.unwrap();assert_eq!(legacy,"Jataí");
}
