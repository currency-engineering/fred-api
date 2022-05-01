//! An API to [FRED Economic Data](https://fred.stlouisfed.org/). [FRED API key](https://fred.stlouisfed.org/docs/api/api_key.html)
//! needs to be stored as the environment variable `FRED_API_KEY`. Only `JSON` response format is
//! implemented.
//!
//! ### Examples
//!
//! To make a single request,
//! ```
//! let series_tags = FredClient::series_tags("JPNCPIALLMINMEI").unwrap();
//! ```
//!
//! To make multiple pipelined requests,
//! ```
//! let iter = FredClientIter::new(vec![1, 2, 3].iter(), |id| category_id(id));
//! let responses = FredClient::pipeline(iter);
//! ```

// We need to be able to specify the return type of a request. To do this the user builds functions
// with arbitrary arguments and arbitrary return types. These functions internally call req() which
// makes the requests and coerce into return type.
//

use anyhow::{anyhow, Context, Error, Result};
use keytree::serialize::{
    KeyTreeString,
    IntoKeyTree,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::{
    env,
    fmt::{Display, self},
    iter::Iterator,
};

// We want to create an iterator by giving it a closure like `|id| category_id(id)`,
// and a Vec<args> (Vec<usize> in this case). What is library side and what is user side?
// We can pass the arguments as a single argument

/// An iterator that makes a group of API requests.
///
/// To make a group of requests we can build a `FredClientIter` from a collection of request
/// arguments and specify a requestion function as a closure, for example
/// ```
/// let iter = FredClientIter::new(vec![1, 2, 3].iter(), |id| category_id(id));
/// ```
/// If there are multiple arguments we can do
/// ```
/// let iter = FredClientIter::new(
///     vec![(1, "tag1"), (2, "tag2")].iter(),
///     |(id, tag)| category_related_tags(id, tag),
/// );
/// ```
pub struct FredClientIter<ArgsIter, F, U>
where
    // Any iterator such as vec![1, 2, 3].iter() in the example above. 
    ArgsIter: Iterator,
    // A function such as `category_id(id) -> Result<Categories>`
    F: Fn(<ArgsIter as Iterator>::Item) -> Result<U>,
{
    args_iter: ArgsIter,
    f: F,
}

impl<ArgsIter, F, U> FredClientIter<ArgsIter, F, U>
where
    ArgsIter: Iterator,
    F: Fn(<ArgsIter as Iterator>::Item) -> Result<U>,
{
    pub fn new(args_iter: ArgsIter, f: F) -> FredClientIter<ArgsIter, F, U>
    {
        FredClientIter { args_iter, f }
    }
}

impl<ArgsIter, F, U> Iterator for FredClientIter<ArgsIter, F, U>
where
    ArgsIter: Iterator,
    F: Fn(<ArgsIter as Iterator>::Item) -> Result<U>,
{
    type Item = Result<U, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.args_iter.next().map(|args| (&self.f)(args))
    }
}

/// Encapsulate all the different types of API request.
pub struct FredClient;

// The functions in this impl should concisely encapsulate information abou the requests together
// with the response type, in a coded form. The functions pass off the plumbing to the req()
// functions.
impl FredClient {
    /// [Get a category](https://fred.stlouisfed.org/docs/api/fred/category.html)
    /// ```
    /// for i in 0..100 {
    ///     match Fred::category(i) {
    ///         Ok(category) => println!("{}\n{}", i, category),
    ///         Err(json_err) => {},
    ///     }
    /// }
    /// ```
    pub fn category(category_id: usize) -> Result<Categories> { 
        req(
            FredRequest::new("category", vec![("category_id", category_id)])?
        )
    }

    /// [Get the child categories for a specified parent category.](https://fred.stlouisfed.org/docs/api/fred/category_children.html)
    pub fn category_children(category_id: usize) -> Result<Categories> {
        req(
            FredRequest::new("category/children", vec![("category_id", category_id)])?
        )
    }
    
    /// [Get the related categories for a category.](https://fred.stlouisfed.org/docs/api/fred/category_related.html)
    pub fn category_related(category_id: usize) -> Result<Categories> {
        req(
            FredRequest::new("category/related", vec![("category_id", category_id)])?
        )
    }
    
    /// [Get the series in a category.](https://fred.stlouisfed.org/docs/api/fred/category_series.html)
    pub fn category_series(category_id: usize) -> Result<CategorySeries> {
        req(
            FredRequest::new("category/series", vec![("category_id", category_id)])?
        )
    }
    
    /// [Get the tags for a category.](https://fred.stlouisfed.org/docs/api/fred/category_tags.html)
    pub fn category_tags(category_id: usize) -> Result<CategoryTags> {
        req(
            FredRequest::new("category/tags", vec![("category_id", category_id)])?
        )
    }
    
    /// [Get the related tags for a category.](https://fred.stlouisfed.org/docs/api/fred/category_related_tags.html)
    pub fn category_related_tags<T: std::fmt::Display>(category_id: T, tag_names: T) -> Result<CategoryRelatedTags> {
        req(
            FredRequest::new(
                "category/related_tags",
                vec![("category_id", category_id), ("tag_names", tag_names)],
            )?
        )
    }

    /// [Get all releases of economic data.](https://fred.stlouisfed.org/docs/api/fred/releases.html)
    pub fn releases() -> Result<Releases> { 
        req(
            FredRequest::new(
                "releases",
                Vec::<(&'static str, String)>::new(),
            )?
        )
    }

    /// [Get release dates for all releases of economic data.](https://fred.stlouisfed.org/docs/api/fred/releases_dates.html)
    pub fn releases_dates() -> Result<ReleaseDates> { 
        req(
            FredRequest::new("releases/dates", Vec::<(&'static str, String)>::new())?
        )
    }

    /// [Get a release of economic data.](https://fred.stlouisfed.org/docs/api/fred/release.html)
    pub fn release(release_id: usize) -> Result<Release> { 
        req(
            FredRequest::new( "release", vec![("release_id", release_id)])?
        )
    }

    /// [Get release dates for a release of economic data.](https://fred.stlouisfed.org/docs/api/fred/release_dates.html)
    pub fn release_dates(release_id: usize) -> Result<ReleaseDates> { 
        req(
            FredRequest::new("release/dates", vec![("release_id", release_id)])?
        )
    }

    /// [Get the series on a release of economic data.](https://fred.stlouisfed.org/docs/api/fred/release_series.html)
    pub fn release_series(release_id: usize) -> Result<ReleaseSeries> { 
        req(
            FredRequest::new("release/series", vec![("release_id", release_id)])?
        )
    }

    /// [Get the sources for a release of economic data.](https://fred.stlouisfed.org/docs/api/fred/release_sources.html)
    pub fn release_sources(release_id: usize) -> Result<ReleaseSources> { 
        req(
            FredRequest::new("release/sources", vec![("release_id", release_id)])?
        )
    }

    /// [Get the tags for a release.](https://fred.stlouisfed.org/docs/api/fred/release_tags.html)
    pub fn release_tags(release_id: usize) -> Result<ReleaseTags> { 
        req(
            FredRequest::new("release/tags", vec![("release_id", release_id)])?
        )
    }

    /// [Get the related tags for a release.](https://fred.stlouisfed.org/docs/api/fred/release_related_tags.html)
    pub fn release_related_tags<T: std::fmt::Display>(release_id: T, tag_names: T) -> Result<ReleaseRelatedTags> { 
        req(
            FredRequest::new(
                "release/related_tags",
                vec![("release_id", release_id), ("tag_names", tag_names)],
            )?
        )
    }

    /// [Get the release tables for a given release.](https://fred.stlouisfed.org/docs/api/fred/release_tables.html)
    pub fn release_tables(release_id: usize) -> Result<ReleaseTables> { 
        req(
            FredRequest::new("release/tables", vec![("release_id", release_id)])?
        )
    }

    /// [Get an economic data series.](https://fred.stlouisfed.org/docs/api/fred/series.html)
    pub fn series(series_id: &str) -> Result<Series> { 
        req(
            FredRequest::new("series", vec![("series_id", series_id)])?
        )
    }

    /// Return the series request as JSON. 
    pub fn series_json(series_id: &str) -> Result<String> {
        req(
            FredRequest::new("series", vec![("series_id", series_id)])?
        )
    }

    /// [Get the categories for an economic data series.](https://fred.stlouisfed.org/docs/api/fred/series_categories.html)
    pub fn series_categories(series_id: &str) -> Result<Categories> { 
        req(
            FredRequest::new("series/categories", vec![("series_id", series_id)])?
        )
    }

    /// [Get the observations or data values for an economic data series.](https://fred.stlouisfed.org/docs/api/fred/series_observations.html)
    pub fn series_observations(series_id: &str) -> Result<SeriesObservations> { 
        req(
            FredRequest::new("series/observations", vec![("series_id", series_id)])?
        )
    }

    /// Return the series_observations request as JSON. 
    pub fn series_observations_json(series_id: &str) -> Result<String> {
        req(
            FredRequest::new("series/observations", vec![("series_id", series_id)])?
        )
    }

    /// [Get the release for an economic data series.](https://fred.stlouisfed.org/docs/api/fred/series_release.html)
    pub fn series_release(series_id: &str) -> Result<SeriesRelease>{ 
        req(
            FredRequest::new("series/release", vec![("series_id", series_id)])?
        )
    }

    /// [Get economic data series that match keywords.](https://fred.stlouisfed.org/docs/api/fred/series_search.html)
    pub fn series_search(search_text: &str) -> Result<SeriesSearch> { 
        req(
            FredRequest::new("series/search", vec![("search_text", search_text)])?
        )
    }

    /// [Get the tags for a series search.](https://fred.stlouisfed.org/docs/api/fred/series_search_tags.html)
    pub fn series_search_tags(series_search_text: &str) -> Result<SeriesSearchTags> { 
        req(
            FredRequest::new("series/search/tags", vec![("series_search_text", series_search_text)])?
        )
    }

    /// [Get the related tags for a series search.](https://fred.stlouisfed.org/docs/api/fred/series_search_related_tags.html)
    pub fn series_search_related_tags(series_search_text: &str, tag_names: &str) -> Result<SeriesSearchRelatedTags> { 
        req(
            FredRequest::new(
                "series/search/related_tags",
                vec![("series_search_text", series_search_text), ("tag_names", tag_names)],
            )?
        )
    }

    /// [Get the tags for an economic data series.](https://fred.stlouisfed.org/docs/api/fred/series_tags.html)
    /// ```
    /// let series_tags = Fred::series_tags("JPNCPIALLMINMEI")
    ///     .unwrap_or_else{|e| eprintln!(e)};
    /// ```
    pub fn series_tags(series_id: &str) -> Result<SeriesTags> { 
        req(
            FredRequest::new("series/tags", vec![("series_id", series_id)])?
        )
    }

    /// [Get economic data series sorted by when observations were updated on the FRED® server.](https://fred.stlouisfed.org/docs/api/fred/series_updates.html)
    pub fn series_updates() -> Result<SeriesUpdates> { 
        req(
            FredRequest::new("series/updates", Vec::<(&'static str, String)>::new())?
        )
    }

    /// [Get the dates in history when a series' data values were revised or new data values were released.](https://fred.stlouisfed.org/docs/api/fred/series_vintagedates.html)
    pub fn series_vintagedates(series_id: &str) -> Result<SeriesVintageDates> { 
        req(
            FredRequest::new("series/vintagedates", vec![("series_id", series_id)])?
        )
    }

    /// [Get all sources of economic data.](https://fred.stlouisfed.org/docs/api/fred/sources.html)
    pub fn sources() -> Result<Sources> { 
        req(
            FredRequest::new("sources", Vec::<(&'static str, String)>::new())?
        )
    }

    /// [Get a source of economic data.](https://fred.stlouisfed.org/docs/api/fred/source.html)
    pub fn source(source_id: usize) -> Result<ReleaseSources> { 
        req(
            FredRequest::new("source", vec![("source_id", source_id)])?
        )
    }

    /// [Get the releases for a source.](https://fred.stlouisfed.org/docs/api/fred/source_releases.html)
    pub fn source_releases(source_id: usize) -> Result<SourceReleases> { 
        req(
            FredRequest::new("source/releases", vec![("source_id", source_id)])?
        )
    }

    /// [Get all tags, search for tags, or get tags by name.](https://fred.stlouisfed.org/docs/api/fred/tags.html)
    pub fn tags() -> Result<Tags> { 
        req(
            FredRequest::new("tags", Vec::<(&'static str, String)>::new())?
        )
    }

    /// [Get the related tags for one or more tags.](https://fred.stlouisfed.org/docs/api/fred/related_tags.html)
    pub fn related_tags(tag_names: &str) -> Result<Tags> { 
        req(
            FredRequest::new("related_tags", vec![("tag_names", tag_names)])?
        )
    }

    /// [Get the series matching tags.](https://fred.stlouisfed.org/docs/api/fred/tags_series.html)
    /// ```
    /// match Fred::tags_series("korea") {
    ///     Ok(tags_series) => println!("{}", tags_series),
    ///     Err(json_err) => println!("{}", json_err),
    /// };
    /// ```
    /// or
    /// ``` 
    /// Fred::tags_series("cpi;usa;nation").unwrap().to_string()
    /// ```
    pub fn tags_series(tag_names: &str) -> Result<TagsSeries> { 
        req(
            FredRequest::new( "tags/series", vec![("tag_names", tag_names)])?
        )
   } 
}

// This is the plumbing for the Fred API request functions. 

// fn req<T, U>(url: &str, keyvals: Vec<(&'static str, T)>) -> Result<U>
// where
//     T: Display,
//     U: DeserializeOwned,
// {
//     let req = FredRequest::new(url, keyvals, Format::Json)?;
// 
//     // Makes the network request.
//     let response = response(req)?;
// 
//     // Coerces to the return type U
//     serde_json::from_str(&response).context(format!("Failed to parse [{}]", response))
// }

// This function has a short name for use in many functions, and handles the coercion into the
// return types of those many functions.
fn req<R, U>(into_req: R) -> Result<U>
where
    U: DeserializeOwned,
    R: IntoRequest,
{
    // Makes the network request.
    let response = response(into_req)?;
    
    // Coerces to the return type U
    serde_json::from_str(&response).context(format!("Failed to parse [{}]", response))
}

/// Construct a request and return the response.
fn response<R: IntoRequest>(into_req: R) -> Result<String> {

    let req: String = into_req.into_request()?;

    let  blocking_response = reqwest::blocking::get(&req)?;

    let response = blocking_response.text_with_charset("utf-8")?;

    let first_line = match response.lines().next() {
        Some(line) => line,
        None => return Err(anyhow!(format!("Http response to [{:?}] was empty.", req))),
    };

    if first_line.contains("error_code") {
        return Err(anyhow!(format!("Http request [{:?}] failed with error [{}].", req, &response)))
    }
    Ok(response)
}

/// Response format can be JSON or XML.
#[derive(Debug)]
pub enum Format {
    /// JSON
    Json,
    /// XML
    Xml,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Json => write!(f, "file_type=json"),
            Format::Xml => write!(f, "file_type=xml"),
        }
    }
}

/// A trait for types that can be converted into an HTTP request string.
pub trait IntoRequest {
    fn into_request(&self) -> Result<String>;

    fn base_url(&self) -> Result<String>;

    fn api_key(&self) -> Result<String>;
}

impl IntoRequest for FredRequest {
    fn into_request(&self) -> Result<String> {
        Ok(
            format!(
                "{}fred/{}?{}&api_key={}&{}",
                self.base_url()?,
                self.url,
                self.concat_keyvals('&'),
                self.api_key()?,
                self.format,
            )
        )
    }

    fn base_url(&self) -> Result<String> {
        Ok(
            "https://api.stlouisfed.org/".into()
        )
    }

    fn api_key(&self) -> Result<String> {
        env::var("FRED_API_KEY").map_err(|_| anyhow!("Expected FRED_API_KEY to be set"))
    }
}

#[derive(Debug)]
pub struct FredRequest {
    url:        String,         // The request category;
    keyvals:    Vec<String>,    // The tags;
    format:     Format,
}

impl FredRequest {

    // The procedure for building a Request involves first setting the fields that are general for
    // all requests, and then later appending key-values that are specific to each request (i.e.
    // each Fred method). This explains the function arguments.
    // pub fn new<T: Display>(url: &str, keyvals: Vec<(&'static str, T)>, format: Format) -> Result<Self> {
    //     let api_key = env::var("FRED_API_KEY").map_err(|_| anyhow!("Expected FRED_API_KEY to be set"))?;
    //     let kvs = keyvals.iter().map(|(key, val)| format!("{}={}", key, val)).collect(); 
    //     Ok(
    //         FredRequest {
    //             url:        url.into(),
    //             keyvals:    kvs,  
    //             format:     format.into(),
    //         }
    //     )
    // }
    
    pub fn new<T: Display>(url: &str, keyvals: Vec<(&'static str, T)>) -> Result<Self> {
        let kvs = keyvals.iter().map(|(key, val)| format!("{}={}", key, val)).collect(); 
        Ok(
            FredRequest {
                url:        url.into(),
                keyvals:    kvs,  
                format:     Format::Json,
            }
        )
    }

    fn concat_keyvals(&self, sep: char) -> String {
        let mut s = String::new();
        for kv in &self.keyvals {
            s.push_str(&kv);
            s.push(sep);
        }
        s.pop();
        s
    }
}

// Response data-structures ///////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct Categories {
    pub categories: Vec<Category>,
}

impl fmt::Display for Categories {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for category in &self.categories {
            s.push_str(&category.to_string());
        };
        write!(f, "{}", s)
    }
}

/// See [Fred docs: /fred/category](https://fred.stlouisfed.org/docs/api/fred/category.html).
///
#[derive(Debug, Deserialize)]
pub struct Category {
    pub id:                         usize,
    pub name:                       String,
    pub parent_id:                  usize,
    pub notes:                      Option<String>,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "id: {}\nname: {}\nparent_id: {}\nnotes: {:?}\n",
            self.id,
            self.name,
            self.parent_id,
            self.notes,
        )
    }
}

/// See [Fred docs: /fred/category/children](https://fred.stlouisfed.org/docs/api/fred/category_children.html).
#[derive(Debug, Deserialize)]
pub struct CategoryChildren {
    pub categories:                 Vec<Category>,
}

/// See [Fred docs: /fred/category/related](https://fred.stlouisfed.org/docs/api/fred/category_related.html).
#[derive(Debug, Deserialize)]
pub struct CategoryRelated {
    #[allow(dead_code)]
    categories:                     Vec<Category>,
}

pub struct SeriesItemsIter<'a> {
    data: &'a SeriesItems,
    count: usize, 
}

impl<'a> Iterator for SeriesItemsIter<'a> {
    type Item = &'a SeriesItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.data.0.len() {
            None
        } else {
            self.count += 1;
            Some(&self.data.0[self.count - 1])
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SeriesItems(Vec<SeriesItem>);

impl SeriesItems {
    pub fn iter<'a>(&'a self) -> SeriesItemsIter {
        SeriesItemsIter {
            data: &self,
            count: 0,
        }
    }

    /// Exclude if includes phrase.
    pub fn exclude_phrases(&self, phrases: Vec<&str>) -> SeriesItems {
        let mut v = Vec::new();
        for series in self.iter() {
            if !phrases.iter().any(|title| series.title.contains(title)) {
                v.push(series.clone());
            }
        }
        SeriesItems(v)
    }

    /// Only include series where title has phrase `phrase`.
    pub fn has_phrase(&self, phrase: &str) -> SeriesItems {
        let mut v = Vec::new();
        for series in self.iter() {
            if series.title.contains(phrase) {
                v.push(series.clone());
            }
        }
        SeriesItems(v)
    }

    /// Include only series with series equal to title.
    pub fn equals_one_of(&self, titles: Vec<&str>) -> SeriesItems {
        let mut v = Vec::new();
        for series in self.iter() {
            if titles.iter().any(|title| *title == series.title) {
                v.push(series.clone());
            }
        }
        SeriesItems(v)
    }

    /// Only include if includes phrase.
    pub fn only_include(&self, phrases: Vec<&str>) -> SeriesItems {
        let mut v = Vec::new();
        for series in &self.0 {
            if phrases.iter().any(|title| series.title.contains(title)) {
                v.push(series.clone());
            }
        }
        SeriesItems(v)
    }

    pub fn inner(&self) -> Vec<SeriesItem> {
        (*self.0).to_vec()
    }
}

impl IntoKeyTree for SeriesItems {

    fn keytree(&self) -> KeyTreeString {
        let mut s = KeyTreeString::new();
        s.push_key(0, "series_items");
        for series_item in self.iter() {
            let sis = series_item.keytree();
            s.push_keytree(1, sis);
        }
        s
    }   
}

impl fmt::Display for SeriesItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = KeyTreeString::new();
        for (i, series) in self.0.iter().enumerate() {
            s.push_keyvalue(0, "series", &i.to_string());
            s.push_keytree(1, series.keytree());
        };
        write!(f, "{}", s)
    }
}

/// See Fred docs
#[derive(Debug, Deserialize)]
pub struct CategorySeries {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub seriess:                    SeriesItems,
}

impl fmt::Display for CategorySeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for (i, series) in self.seriess.0.iter().enumerate() {
            s.push_str(&i.to_string());
            s.push('\n');
            s.push_str(&series.to_string());
            s.push('\n');
        };
        write!(f, "{}", s)
    }
}

/// See [Fred docs: /fred/category/tags](https://fred.stlouisfed.org/docs/api/fred/category_tags.html).
#[derive(Debug, Deserialize)]
pub struct CategoryTags {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub tags:                       Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name:                       String,
    pub group_id:                   String,
    pub notes:                      Option<String>,
    pub created:                    String,
    pub popularity:                 isize,
    pub series_count:               isize,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "name: {}\nnotes: {:?}\nseries_count: {}",
            self.name,
            self.notes,
            self.series_count
        )
    }
}

/// See [Fred docs: /fred/category/related_tags](https://fred.stlouisfed.org/docs/api/fred/category_related_tags.html).
#[derive(Debug, Deserialize)]
pub struct CategoryRelatedTags {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub tags:                       Vec<Tag>,
}

/// See [Fred docs: /fred/releases](https://fred.stlouisfed.org/docs/api/fred/releases.html).
#[derive(Debug, Deserialize)]
pub struct Releases {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub releases:                   Vec<ReleaseItem>,
}

/// See [Fred docs: /fred/release](https://fred.stlouisfed.org/docs/api/fred/release.html).
#[derive(Debug, Deserialize)]
pub struct Release {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub releases:                   Vec<ReleaseItem>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseItem {
    pub id:                         isize,
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub name:                       String,
    pub press_release:              bool,
    pub link:                       Option<String>,
}

/// See [Fred docs: /fred/release/dates](https://fred.stlouisfed.org/docs/api/fred/release_dates.html).
#[derive(Debug, Deserialize)]
pub struct ReleasesDates {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub release_dates:              Vec<ReleaseDate>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseDates {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub release_dates:              Vec<ReleaseDateItem>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseDateItem {
    pub release_id:                 isize,
    pub date:                       String,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseDate {
    pub release_id:                 isize,
    pub release_name:               String,
    pub date:                       String,
}

/// See [Fred docs: /fred/release/series](https://fred.stlouisfed.org/docs/api/fred/release_series.html).
#[derive(Debug, Deserialize)]
pub struct ReleaseSeries {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub seriess:                    SeriesItems,
}

/// See [Fred docs: /fred/release/sources](https://fred.stlouisfed.org/docs/api/fred/release_sources.html).
#[derive(Debug, Deserialize)]
pub struct ReleaseSources {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub sources:                    Vec<SourceItem>,
}

#[derive(Debug, Deserialize)]
pub struct SourceItem {
    pub id:                         isize,
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub name:                       String,
    pub link:                       Option<String>,
}

/// See [Fred docs: /fred/release/tags](https://fred.stlouisfed.org/docs/api/fred/release_tags.html).
#[derive(Debug, Deserialize)]
pub struct ReleaseTags {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub tags:                       Vec<TagItem>,
}

#[derive(Debug, Deserialize)]
pub struct TagItem {
    pub name:                       String,
    pub group_id:                   String,
    pub notes:                      Option<String>,
    pub created:                    String,
    pub popularity:                 isize,
    pub series_count:               isize,
}

impl fmt::Display for TagItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "name: {}\ngroup_id: {}\nnotes: {:?}\n",
            self.name,
            self.group_id,
            self.notes,
        )
    }
}

/// See [Fred docs: /fred/release/related_tags](https://fred.stlouisfed.org/docs/api/fred/release_related_tags.html).
#[derive(Debug, Deserialize)]
pub struct ReleaseRelatedTags {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub tags:                       Vec<TagItem>,
}

/// See [Fred docs: /fred/release/related_tags](https://fred.stlouisfed.org/docs/api/fred/release_related_tags.html).
#[derive(Debug, Deserialize)]
pub struct ReleaseTables {
    pub name:                       String,
    pub element_id:                 isize,
    pub release_id:                 String,
    pub elements:                   Vec<ReleaseKeyVal>,
}

/// See [Fred docs: /fred/release/related_tags](https://fred.stlouisfed.org/docs/api/fred/release_related_tags.html).
#[derive(Debug, Deserialize)]
pub struct ReleaseKeyVal {
    pub key:                        isize,
    pub value:                      ReleaseElement,
}

/// See [Fred docs: /fred/release/related_tags](https://fred.stlouisfed.org/docs/api/fred/release_related_tags.html).
#[derive(Debug, Deserialize)]
pub struct ReleaseElement {
    pub element_id:                 isize,
    pub release_id:                 String,
    pub series_id:                  String,
    pub parent_id:                  String,
    pub line:                       String,
    #[serde(rename = "type")] 
    pub ty:                         String,
    pub name:                       String,
    pub level:                      String,
    pub children:                   Vec<ReleaseElement>,
}

#[derive(Debug, Deserialize)]
pub struct Series {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub seriess:                    SeriesItems,
}

impl IntoKeyTree for Series {
    fn keytree(&self) -> KeyTreeString {
        let mut s = KeyTreeString::new();
        s.push_key(0, "series");
        s.push_keyvalue(1, "realtime_start", &self.realtime_start);
        s.push_keyvalue(1, "realtime_end", &self.realtime_end);
        s.push_keytree(1, self.seriess.keytree());
        s
    } 
} 

impl fmt::Display for Series {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.keytree();
        writeln!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SeriesItem {
    pub id:                         String,
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub title:                      String,
    pub observation_start:          String,
    pub observation_end:            String,
    pub frequency:                  String,
    pub units:                      String,
    pub units_short:                String,
    pub seasonal_adjustment:        String,
    pub seasonal_adjustment_short:  String,
    pub last_updated:               String,
    pub popularity:                 isize,              // ignore in string representation
    pub group_popularity:           Option<isize>,      // ignore in string representation
    pub notes:                      Option<String>,
}

impl SeriesItem {
    pub fn keytree(&self) -> KeyTreeString {
        let mut s = KeyTreeString::new();
        s.push_key(0, "series_item");
        s.push_keyvalue(1, "id", &self.id);
        s.push_keyvalue(1, "realtime_start", &self.realtime_start);
        s.push_keyvalue(1, "realtime_end", &self.realtime_end);
        s.push_keyvalue(1, "title", &self.title);
        s.push_keyvalue(1, "observation_start", &self.observation_start);
        s.push_keyvalue(1, "observation_end", &self.observation_end);
        s.push_keyvalue(1, "frequency", &self.frequency);
        s.push_keyvalue(1, "units", &self.units);
        s.push_keyvalue(1, "units_short", &self.units_short);
        s.push_keyvalue(1, "seasonal_adjustment", &self.seasonal_adjustment);
        s.push_keyvalue(1, "last_updated", &self.last_updated);
        s.push_keyvalue(1, "notes", "(see JSON data for notes)");
        s
    }

    pub fn tags(&self) -> String {
        FredClient::series_tags(&self.id).unwrap().one_line()
    }
}

impl fmt::Display for SeriesItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.keytree();
        writeln!(f, "{}", s)
    }
}

/// See [Fred docs: /fred/series/categories](https://fred.stlouisfed.org/docs/api/fred/series_categories.html).
#[derive(Debug, Deserialize)]
pub struct SeriesCategories {
    pub categories:                 Vec<CategoryItem>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryItem {
    pub id:                         String,
    pub name:                       String,
    pub parent_id:                  isize,
}

#[derive(Debug, Deserialize)]
pub struct Observations(Vec<Observation>);

impl Observations {
    pub fn iter<'a>(&'a self) -> ObservationsIter {
        ObservationsIter {
            data: &self,
            count: 0,
        }
    }
}

impl fmt::Display for Observations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for obs in self.iter() {
            s.push_str(&obs.to_string());
            s.push('\n');
        }
        s.pop();
        write!(f, "{}", s)
    }
}

pub struct ObservationsIter<'a> {
    data: &'a Observations,
    count: usize, 
}

impl<'a> Iterator for ObservationsIter<'a> {
    type Item = &'a Observation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.data.0.len() {
            None
        } else {
            self.count += 1;
            Some(&self.data.0[self.count - 1])
        }
    }
}

/// See [Fred docs: /fred/series/observations](https://fred.stlouisfed.org/docs/api/fred/series_observations.html).
#[derive(Debug, Deserialize)]
pub struct SeriesObservations {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub observation_start:          String, 
    pub observation_end:            String, 
    pub units:                      String, 
    pub output_type:                isize, 
    pub file_type:                  String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub observations:               Observations, 
}

/// See [Fred docs: /fred/series/observations](https://fred.stlouisfed.org/docs/api/fred/series_observations.html).
#[derive(Debug, Deserialize)]
pub struct Observation {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub date:                       String, 
    pub value:                      String, 
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.date, self.value)
    }
}

/// See [Fred docs: /fred/series/release](https://fred.stlouisfed.org/docs/api/fred/series_release.html).
#[derive(Debug, Deserialize)]
pub struct SeriesRelease {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub releases:                   Vec<ReleaseItem>,
}

/// See [Fred docs: /fred/series/search](https://fred.stlouisfed.org/docs/api/fred/series_search.html).
#[derive(Debug, Deserialize)]
pub struct SeriesSearch {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub seriess:                    SeriesItems,
}

#[derive(Debug, Deserialize)]
pub struct SeriesSearchTags {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub tags:                       Vec<TagItem>,
}

/// See [Fred docs: /fred/series/search/related_tags](https://fred.stlouisfed.org/docs/api/fred/series_search_related_tags.html).
#[derive(Debug, Deserialize)]
pub struct SeriesSearchRelatedTags {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub tags:                       Vec<TagItem>,
}

/// See [Fred docs: /fred/series/tags](https://fred.stlouisfed.org/docs/api/fred/series_tags.html).
#[derive(Debug, Deserialize)]
pub struct SeriesTags {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub tags:                       Vec<TagItem>,
}

impl SeriesTags {
    pub fn one_line(&self) -> String {
        let mut s = String::new();
        for tag in self.tags.iter() {
            s.push_str(&tag.name);
            s.push_str(", ");
        };
        s  
    }
}

impl fmt::Display for SeriesTags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for (i, tags) in self.tags.iter().enumerate() {
            s.push_str(&i.to_string());
            s.push('\n');
            s.push_str(&tags.to_string());
            s.push('\n');
        };
        write!(f, "{}", s)
    }
}

/// See [Fred docs: /fred/series/updates](https://fred.stlouisfed.org/docs/api/fred/series_updates.html).
#[derive(Debug, Deserialize)]
pub struct SeriesUpdates {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub filter_variable:            String, 
    pub filter_value:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub seriess:                    SeriesItems,
}

/// See [Fred docs: /fred/series/vintage_dates](https://fred.stlouisfed.org/docs/api/fred/series_vintagedates.html).
#[derive(Debug, Deserialize)]
pub struct SeriesVintageDates {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub vintage_dates:              Vec<String>,
}

/// See [Fred docs: /fred/sources](https://fred.stlouisfed.org/docs/api/fred/sources.html).
#[derive(Debug, Deserialize)]
pub struct Sources {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize,
    pub limit:                      isize, 
    pub sources:                    Vec<SourceItem>,
}

/// See [Fred docs: /fred/source/releases](https://fred.stlouisfed.org/docs/api/fred/source_releases.html).
#[derive(Debug, Deserialize)]
pub struct SourceReleases {
    pub realtime_start:             String, 
    pub realtime_end:               String, 
    pub order_by:                   String, 
    pub sort_order:                 String, 
    pub count:                      isize, 
    pub offset:                     isize, 
    pub limit:                      isize,
    pub releases:                   Vec<ReleaseItem>,
}


/// See [Fred docs: /fred/tags](https://fred.stlouisfed.org/docs/api/fred/tags.html).
#[derive(Debug, Deserialize)]
pub struct Tags {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize,
    pub limit:                      isize,
    pub tags:                       Vec<Tag>,
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tags = String::new();
        for (i, tag) in self.tags.iter().enumerate() {
            tags.push_str(&i.to_string());
            tags.push('\n');
            tags.push_str(&tag.to_string());
            tags.push('\n');
            tags.push('\n');
        };
        write!(f, "{}", tags)
    }
}

/// See [Fred docs: /fred/tags/series](https://fred.stlouisfed.org/docs/api/fred/tags_series.html).
#[derive(Debug, Deserialize)]
pub struct TagsSeries {
    pub realtime_start:             String,
    pub realtime_end:               String,
    pub order_by:                   String,
    pub sort_order:                 String,
    pub count:                      isize,
    pub offset:                     isize, 
    pub limit:                      isize, 
    pub seriess:                    SeriesItems,
}

impl TagsSeries {
    pub fn series_titles(&self) -> String {
        let mut s = String::new();
        for series in &self.seriess.0 {
            s.push_str(&series.title);
            s.push('\n');
        }
        s
    }

    pub fn series(self) -> SeriesItems {
        self.seriess
    }
}

impl Display for TagsSeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut series = String::new();
        for (i, s) in self.seriess.0.iter().enumerate() {
            series.push_str(&format!("{}\n", i));
            series.push_str(&s.to_string());
            series.push('\n');
        };
        write!(f, "{}", series)
    }
}

mod test {
    use crate::{
        FredClient,
        FredClientIter,
    };

    #[test]
    fn fred_client_iter_works() {
        let iter = FredClientIter::new(vec![1, 2, 3].iter(), |id| FredClient::category(*id));
    }
}
