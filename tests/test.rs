use fred_api::{
    Format,
    Fred,
    Request,
};

#[test]
fn series_tags_request() {

    let req = Request::new("series/tags", vec![("series_id", "JPNCPIALLMINMEI")], Format::Json)
        .unwrap()
        .to_string();

    assert!(req.starts_with("https://api.stlouisfed.org/fred/series/tags?series_id=JPNCPIALLMINMEI&api_key="));
    assert!(req.ends_with("&file_type=json"));
}

#[test]
fn test_series_tags() {
    let series_tags = Fred::series_tags("JPNCPIALLMINMEI").unwrap();
    assert_eq!(series_tags.order_by, "series_count");
}

// #[test]
// fn category() {
//     Fred::category(1);
// }

// #[test]
// fn category_children() {
//     wait();
//     Fred::category_children(1);
// }
// 
// #[test]
// fn category_related() { 
//     wait();
//     Fred::category_related(4);
// }
// 
// #[test]
// fn category_series() {
//     wait();
//     Fred::category_series(4); }
// 
// #[test]
// fn category_tags() {
//     wait();
//     Fred::category_tags(4);
// }
// 
// #[test]
// fn category_related_tags() {
//     wait();
//     Fred::category_related_tags(4, "potatoes");
// }
// 
// #[test]
// fn releases() {
//     wait();
//     Fred::releases();
// }
// 
// #[test]
// fn releases_date() {
//     wait();
//     Fred::releases_dates();
// }
// 
// #[test]
// fn release() {
//     wait();
//     Fred::release(478);
// }
// 
// #[test]
// fn release_date() {
//     wait();
//     Fred::release_dates(478);
// }
// 
// #[test]
// fn release_series() {
//     wait();
//     Fred::release_series(478);
// }
// 
// #[test]
// fn release_sources() {
//     wait();
//     Fred::release_sources(478);
// }
// 
// #[test]
// fn release_tags() {
//     wait();
//     Fred::release_tags(478);
// }
// 
// #[test]
// fn release_related_tags() {
//     wait();
//     Fred::release_related_tags(478, "potatoes");
// }
// 
// // TODO Fred::release_tables(478);
// 
// #[test]
// fn series() {
//     wait();
//     Fred::series("LRUNTTTTAUM156S");
// }
// 
// #[test]
// fn series_categories() {
//     wait();
//     Fred::series_categories("LRUNTTTTAUM156S");
// }
// 
// #[test]
// fn series_observations() {
//     wait();
//     Fred::series_observations("LRUNTTTTAUM156S");
// }
// 
// #[test]
// fn series_release() {
//     wait();
//     Fred::series_release("LRUNTTTTAUM156S");
// }
// 
// #[test]
// fn series_search() {
//     wait();
//     Fred::series_search("unemployment");
// }
// 
// #[test]
// fn series_search_tags() {
//     wait();
//     Fred::series_search_tags("unemployment");
// }
// 
// #[test]
// fn series_search_related_tags() {
//     wait();
//     Fred::series_search_related_tags("unemployment", "rate");
// }
// 
// #[test]
// fn series_tags() {
//     wait();
//     Fred::series_tags("LRUNTTTTAUM156S");
// }
// 
// #[test]
// fn series_updates() {
//     wait();
//     Fred::series_updates();
// }
// 
// #[test]
// fn series_vintagedates() {
//     wait();
//     Fred::series_vintagedates("LRUNTTTTAUM156S");
// }
// 
// #[test]
// fn sources() {
//     wait();
//     Fred::sources();
// }
// 
// #[test]
// fn source() {
//     wait();
//     Fred::source(1);
// }
// 
// #[test]
// fn source_releases() {
//     wait();
//     Fred::source_releases(1);
// }
// 
// #[test]
// fn tags() {
//     wait();
//     Fred::tags();
// }
// 
// #[test]
// fn related_tags() {
//     wait();
//     Fred::related_tags("unemployment");
// }
// 
// #[test]
// fn tags_series() {
//     wait();
//     Fred::tags_series("unemployment");
// }
// 
// fn wait() {
//     let r = rand::random::<u8>();
//     sleep(Duration::from_secs((r / 5).into()));
// }
