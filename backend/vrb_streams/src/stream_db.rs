use chrono::{DateTime, Utc};
use vrb_db::db::DbPool2;

use crate::stream_models::{
    CreateStreamAndTags, ModifyStreamAndTags, SearchStreamAndTags, SearchStreamDate, SearchStreamTag, StreamAndTags, StreamTag,
};

pub trait StreamDb {
    /// Add a new entry (stream, tags).
    #[rustfmt::skip]
    fn create_stream_and_tags(
        &self, create_stream: CreateStreamAndTags,
    ) -> impl std::future::Future<Output = Result<Option<StreamAndTags>, String>> + Send;

    /// Modify an entity (stream, tags).
    #[rustfmt::skip]
    fn modify_stream_and_tags(
        &self, id: i32, user_id: i32, modify_stream: ModifyStreamAndTags,
    ) -> impl std::future::Future<Output = Result<Option<StreamAndTags>, String>> + Send;

    /// Delete an entity (stream, tags).
    #[rustfmt::skip]
    fn delete_stream_and_tags(
        &self, id: i32, user_id: i32,
    ) -> impl std::future::Future<Output = Result<Option<StreamAndTags>, String>> + Send;

    /// Get an entity from "streams" and "stream_tags" by ID.
    fn get_stream_and_tags(&self, id: i32) -> impl std::future::Future<Output = Result<Option<StreamAndTags>, String>> + Send;

    /// Get an entity with "live" from "streams" and "stream_tags" by user_id.
    #[rustfmt::skip]
    fn get_stream_and_tags_in_live(
        &self, user_id: i32, exclude_id: i32,
    ) -> impl std::future::Future<Output = Result<Option<StreamAndTags>, String>> + Send;

    /// Filter entities (stream and tags) by the "SearchStream" search parameter.
    #[rustfmt::skip]
    fn filter_stream_and_tags_by_pages(
        &self, search_stream: SearchStreamAndTags,
    ) -> impl std::future::Future<Output = Result<(u32, Vec<StreamAndTags>), String>> + Send;

    /// Filtering objects (stream dates) over a long interval (month) by parameters.
    #[rustfmt::skip]
    fn filter_stream_dates(
        &self, search_stream: SearchStreamDate
    ) -> impl std::future::Future<Output = Result<Vec<DateTime<Utc>>, String>> + Send;

    /// Get an entities (stream_tags) by the "SearchTag" search parameter.
    #[rustfmt::skip]
    fn get_stream_tags(
        &self, search_stream: SearchStreamTag
    ) -> impl std::future::Future<Output = Result<(u32, u32, Vec<StreamTag>), String>> + Send;
}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_stream_db_app(db_pool: &DbPool2) -> impls::StreamDbApp {
    impls::StreamDbApp::new(db_pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_stream_db_app(_: &DbPool2) -> tests::StreamDbApp {
    tests::StreamDbApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use chrono::{DateTime, Utc};
    use log::{Level::Info, info, log_enabled};
    use sqlx;
    use vrb_db::db::DbPool2;

    use crate::stream_db::StreamDb;
    use crate::stream_models::{
        CountStreamAndTags, CreateStreamAndTags, ModifyStreamAndTags, SEARCH_STREAM_AND_TAGS_LIMIT, SEARCH_STREAM_AND_TAGS_PAGE,
        SEARCH_STREAM_TAGS_LIMIT, SEARCH_STREAM_TAGS_PAGE, SearchStreamAndTags, SearchStreamDate, SearchStreamTag, StartStreamAndTags,
        StreamAndTags, StreamTag,
    };

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct StreamDbApp {
        pub db_pool: DbPool2,
    }

    impl StreamDbApp {
        pub fn new(db_pool: &DbPool2) -> Self {
            StreamDbApp {
                db_pool: db_pool.to_owned(),
            }
        }
    }

    impl StreamDb for StreamDbApp {
        /// Add a new entry (stream, tags).
        async fn create_stream_and_tags(&self, create_stream: CreateStreamAndTags) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<StreamAndTags> = sqlx::query_as(
                "SELECT id, user_id, title, descript, logo, starttime, live, state, started, paused, stopped, \"source\", \
                created_at, updated_at, tags, old_logo \
                FROM create_stream_and_stream_tag($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) \
                LIMIT 1",
            )
            .bind(create_stream.user_id)
            .bind(create_stream.title)
            .bind(create_stream.descript)
            .bind(create_stream.logo)
            .bind(create_stream.starttime)
            .bind(create_stream.state)
            .bind(create_stream.started)
            .bind(create_stream.paused)
            .bind(create_stream.stopped)
            .bind(create_stream.source)
            .bind(create_stream.tags)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("create_stream_and_stream_tag: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_stream_and_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Modify an entity (stream, tags).
        #[rustfmt::skip]
        async fn modify_stream_and_tags(
            &self, id: i32, user_id: i32, modify_stream: ModifyStreamAndTags,
        ) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<StreamAndTags> = sqlx::query_as(
                "SELECT id, user_id, title, descript, logo, starttime, live, state, started, paused, stopped, \"source\", \
                created_at, updated_at, tags, old_logo \
                FROM modify_stream_and_stream_tag($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) \
                LIMIT 1",
            )
            .bind(id)
            .bind(user_id)
            .bind(modify_stream.title)
            .bind(modify_stream.descript)
            .bind(modify_stream.logo)
            .bind(modify_stream.starttime)
            .bind(modify_stream.state)
            .bind(modify_stream.started)
            .bind(modify_stream.paused)
            .bind(modify_stream.stopped)
            .bind(modify_stream.source)
            .bind(modify_stream.tags)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("modify_stream_and_stream_tag: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_stream_and_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Delete an entity (stream, tags).
        async fn delete_stream_and_tags(&self, id: i32, user_id: i32) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<StreamAndTags> = sqlx::query_as(
                "SELECT id, user_id, title, descript, logo, starttime, live, state, started, paused, stopped, \"source\", \
                created_at, updated_at, tags, old_logo \
                FROM delete_stream_and_stream_tag($1,$2) \
                LIMIT 1",
            )
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("delete_stream_and_stream_tag: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_stream_and_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Get an entity from "streams" and "stream_tags" by ID.
        async fn get_stream_and_tags(&self, id: i32) -> Result<Option<StreamAndTags>, String> {
            let timer: Option<tm> = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<StreamAndTags> = sqlx::query_as(
                "SELECT id, user_id, title, descript, logo, starttime, live, state, started, paused, stopped, \"source\", \
                created_at, updated_at, tags, old_logo \
                FROM get_stream_and_stream_tag($1) \
                LIMIT 1",
            )
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("get_stream_and_stream_tag: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_stream_and_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Get an entity with "live" from "streams" and "stream_tags" by user_id.
        async fn get_stream_and_tags_in_live(&self, user_id: i32, exclude_id: i32) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<StreamAndTags> = sqlx::query_as(
                "SELECT s.id, s.user_id, s.title, s.descript, s.logo, s.starttime, s.live, s.state, \
                s.started, s.paused, s.stopped, s.\"source\", s.created_at, s.updated_at, \
                ARRAY[]::VARCHAR[] AS tags, NULL AS old_logo \
                FROM streams s, \
                  filter_stream_and_stream_tag_by_pages_ids($1,true,null,null,null,null,null,2,null) f \
                WHERE s.id = f.id AND s.id != $2 \
                LIMIT 1",
            )
            .bind(user_id)
            .bind(exclude_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("(1)filter_stream_and_stream_tag_by_pages_ids: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_stream_and_tags_in_live() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Filter entities (stream and tags) by the "SearchStream" search parameter.
        async fn filter_stream_and_tags_by_pages(&self, search_stream: SearchStreamAndTags) -> Result<(u32, Vec<StreamAndTags>), String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let page: i32 = search_stream.page.unwrap_or(SEARCH_STREAM_AND_TAGS_PAGE).try_into().unwrap();
            let limit: i32 = search_stream.limit.unwrap_or(SEARCH_STREAM_AND_TAGS_LIMIT).try_into().unwrap();
            let offset: i32 = (page - 1) * limit;
            let filter = search_stream.filter.clone().map(|v| v.to_string());

            let stream_and_tags_list: Vec<StreamAndTags> = sqlx::query_as(
                "SELECT id, user_id, title, descript, logo, starttime, live, state, started, paused, stopped, \"source\", \
                created_at, updated_at, tags, old_logo \
                FROM filter_stream_and_stream_tag_by_pages_list($1,$2,$3,$4,$5,$6,$7,$8,$9) \
                LIMIT 500",
            )
            .bind(search_stream.user_id)
            .bind(search_stream.live)
            .bind(filter.clone())
            .bind(search_stream.starttime)
            .bind(search_stream.finishtime)
            .bind(search_stream.sort_desc)
            .bind(search_stream.tag.clone())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("filter_stream_and_stream_tag_by_pages_list: {}", e.to_string()))?;

            let count_stream_and_tags: CountStreamAndTags =
                sqlx::query_as("SELECT * FROM filter_stream_and_stream_tag_by_pages_count($1,$2,$3,$4,$5,$6) as cnt LIMIT 1")
                    .bind(search_stream.user_id)
                    .bind(search_stream.live)
                    .bind(filter)
                    .bind(search_stream.starttime)
                    .bind(search_stream.finishtime)
                    .bind(search_stream.tag)
                    .fetch_one(&self.db_pool)
                    .await
                    .map_err(|e| format!("filter_stream_and_stream_tag_by_pages_count: {}", e.to_string()))?;

            let count: u32 = count_stream_and_tags.cnt.try_into().unwrap();

            if let Some(timer) = timer {
                info!("filter_stream_and_tags_by_pages() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok((count, stream_and_tags_list))
        }

        /// Filtering objects (stream dates) over a long interval (month) by parameters.
        async fn filter_stream_dates(&self, search_stream: SearchStreamDate) -> Result<Vec<DateTime<Utc>>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let stream_date_list: Vec<StartStreamAndTags> = sqlx::query_as(
                "SELECT s.starttime AS start \
                FROM streams s, \
                  filter_stream_and_stream_tag_by_pages_ids($1,null,'period',$2,$3,null,null,null,null) f \
                WHERE s.id = f.id \
                LIMIT 1000",
            )
            .bind(search_stream.user_id)
            .bind(search_stream.start)
            .bind(search_stream.finish)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("filter_stream_and_stream_tag_by_pages_ids: {}", e.to_string()))?;

            let list: Vec<DateTime<Utc>> = stream_date_list.into_iter().map(|v| v.start).collect();

            if let Some(timer) = timer {
                info!("filter_stream_dates() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(list)
        }

        /// Get an entities (stream_tags) by the "SearchTag" search parameter.
        async fn get_stream_tags(&self, search_stream: SearchStreamTag) -> Result<(u32, u32, Vec<StreamTag>), String> {
            let timer: Option<tm> = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let sort_column = search_stream.sort_column.clone().map(|v| v.to_string());
            let page: u32 = search_stream.page.unwrap_or(SEARCH_STREAM_TAGS_PAGE);
            let limit: u32 = search_stream.limit.unwrap_or(SEARCH_STREAM_TAGS_LIMIT);
            let offset: u32 = (page - 1) * limit;
            let limit2: i32 = limit.try_into().unwrap();
            let offset2: i32 = offset.try_into().unwrap();

            let stream_tag_list: Vec<StreamTag> = sqlx::query_as(
                "SELECT id, \"name\", count_links \
                FROM get_stream_tags($1,$2,$3,$4) \
                LIMIT 1000",
            )
            .bind(sort_column)
            .bind(search_stream.sort_desc)
            .bind(limit2)
            .bind(offset2)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("get_stream_tags: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_stream_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok((limit, page, stream_tag_list))
        }
    }
}

// ** **

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::cmp::Ordering;

    use actix_web::web;
    use chrono::{DateTime, Utc};
    use vrb_authent::user_db::tests::{USER_IDS, USER1_ID};
    use vrb_db::enm_stream_state::StreamState;

    use crate::config_strm;
    use crate::stream_db::StreamDb;
    use crate::stream_models::{
        CreateStreamAndTags, FilterStream, ModifyStreamAndTags, SEARCH_STREAM_AND_TAGS_LIMIT, SEARCH_STREAM_AND_TAGS_PAGE,
        SEARCH_STREAM_TAGS_LIMIT, SEARCH_STREAM_TAGS_PAGE, SearchStreamAndTags, SearchStreamDate, SearchStreamTag, StreamAndTags,
        StreamTag, StreamTagSortColumn,
    };

    pub const STREAM_ID: i32 = 1400;
    pub const STREAM_TAG_ID: i32 = 1500;
    pub const TAG_NAME: &str = "tag";

    #[derive(Debug, Clone)]
    pub struct StreamDbApp {
        pub stream_info_vec: Vec<StreamAndTags>,
    }

    impl StreamDbApp {
        /// Create a new instance.
        pub fn new() -> Self {
            StreamDbApp {
                stream_info_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified `stream` list.
        #[cfg(test)]
        pub fn create(stream_vec: &[StreamAndTags]) -> Self {
            let mut stream_info_vec: Vec<StreamAndTags> = Vec::new();

            for (idx, stream) in stream_vec.iter().enumerate() {
                let mut stream2 = stream.clone();
                stream2.live = StreamState::is_live(stream.state);
                let delta: i32 = idx.try_into().unwrap();
                stream2.id = STREAM_ID + delta;
                stream_info_vec.push(stream2);
            }
            StreamDbApp { stream_info_vec }
        }
    }

    impl StreamDb for StreamDbApp {
        /// Add a new entry (stream, tags).
        async fn create_stream_and_tags(&self, create_stream: CreateStreamAndTags) -> Result<Option<StreamAndTags>, String> {
            let len: i32 = self.stream_info_vec.len().try_into().unwrap(); // convert usize as i32

            let mut stream: StreamAndTags = create_stream.into();
            stream.id = STREAM_ID + len;

            Ok(Some(stream))
        }

        /// Modify an entity (stream, tags).
        async fn modify_stream_and_tags(
            &self,
            id: i32,
            user_id: i32,
            modify_stream: ModifyStreamAndTags,
        ) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self
                .stream_info_vec
                .iter()
                .find(|stream| stream.id == id && stream.user_id == user_id)
                .map(|stream| stream.clone());

            if let Some(stream_info) = opt_stream_info {
                let stream = modify_stream.merge(stream_info);
                Ok(Some(stream))
            } else {
                Ok(None)
            }
        }

        /// Delete an entity (stream, tags).
        async fn delete_stream_and_tags(&self, id: i32, user_id: i32) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self
                .stream_info_vec
                .iter()
                .find(|stream| stream.id == id && stream.user_id == user_id)
                .map(|stream| stream.clone());

            Ok(opt_stream_info)
        }

        /// Get an entity from "streams" and "stream_tags" by ID.
        async fn get_stream_and_tags(&self, id: i32) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self.stream_info_vec.iter().find(|stream| stream.id == id).map(|stream| stream.clone());

            Ok(opt_stream_info)
        }

        /// Get an entity with "live" from "streams" and "stream_tags" by user_id.
        async fn get_stream_and_tags_in_live(&self, user_id: i32, exclude_id: i32) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self
                .stream_info_vec
                .iter()
                .find(|stream| stream.live == true && stream.user_id == user_id && stream.id != exclude_id)
                .map(|stream| stream.clone());

            if let Some(stream_info) = opt_stream_info {
                let mut stream = stream_info.clone();
                stream.tags = vec![];
                Ok(Some(stream))
            } else {
                Ok(None)
            }
        }

        /// Filter entities (stream and tags) by the "SearchStream" search parameter.
        async fn filter_stream_and_tags_by_pages(&self, search: SearchStreamAndTags) -> Result<(u32, Vec<StreamAndTags>), String> {
            let mut streams_info: Vec<StreamAndTags> = vec![];

            let future_start = if let Some(FilterStream::Future) = search.filter {
                search.starttime
            } else {
                None
            };
            let past_start = if let Some(FilterStream::Past) = search.filter {
                search.starttime
            } else {
                None
            };
            let period_start = if let Some(FilterStream::Period) = search.filter {
                search.starttime
            } else {
                None
            };
            let period_finish = if let Some(FilterStream::Period) = search.filter {
                search.finishtime
            } else {
                None
            };
            let search_tag = search.tag.unwrap_or_default();

            for stream in self.stream_info_vec.iter() {
                if search.user_id.is_some() && stream.user_id != search.user_id.unwrap() {
                    continue;
                }
                if search.live.is_some() && stream.live != search.live.unwrap() {
                    continue;
                }
                if future_start.is_some() {
                    // future_start <= stream.starttime
                    if stream.starttime < future_start.unwrap() {
                        continue;
                    }
                } else if past_start.is_some() {
                    // past_start < stream.starttime
                    if stream.starttime >= past_start.unwrap() {
                        continue;
                    }
                } else if period_start.is_some() && period_finish.is_some() {
                    // period_start <= stream.starttime <= period_finish
                    if stream.starttime < period_start.unwrap() || period_finish.unwrap() < stream.starttime {
                        continue;
                    }
                }
                if search_tag.len() > 0 && !stream.tags.contains(&search_tag) {
                    continue;
                }

                streams_info.push(stream.clone());
            }

            let is_starttime_desc = search.sort_desc.unwrap_or(false);

            streams_info.sort_by(|a, b| {
                let mut result = if !is_starttime_desc {
                    a.starttime.partial_cmp(&b.starttime).unwrap_or(Ordering::Equal)
                } else {
                    b.starttime.partial_cmp(&a.starttime).unwrap_or(Ordering::Equal)
                };
                if result == Ordering::Equal {
                    result = a.id.partial_cmp(&b.id).unwrap_or(Ordering::Equal);
                }

                result
            });

            let amount = streams_info.len();
            let page = search.page.unwrap_or(SEARCH_STREAM_AND_TAGS_PAGE);
            let limit = search.limit.unwrap_or(SEARCH_STREAM_AND_TAGS_LIMIT);
            let min_idx = (page - 1) * limit;
            let max_idx = min_idx + limit;
            let mut idx = 0;
            streams_info.retain(|_| {
                let res = min_idx <= idx && idx < max_idx;
                idx += 1;
                res
            });

            let count: u32 = amount.try_into().unwrap();
            let streams: Vec<StreamAndTags> = streams_info.iter().map(|v| v.clone()).collect();

            Ok((count, streams))
        }

        /// Filtering objects (stream dates) over a long interval (month) by parameters.
        async fn filter_stream_dates(&self, search_stream: SearchStreamDate) -> Result<Vec<DateTime<Utc>>, String> {
            let search = SearchStreamAndTags::new(
                Some(search_stream.user_id),
                Some(FilterStream::Period),
                Some(search_stream.start),
                Some(search_stream.finish),
            );

            let res = self.filter_stream_and_tags_by_pages(search).await;

            let result: Vec<DateTime<Utc>> = if let Ok((_count, streams)) = res {
                streams.into_iter().map(|v| v.starttime).collect()
            } else {
                vec![]
            };

            Ok(result)
        }

        /// Get an entities (stream_tags) by the "SearchTag" search parameter.
        async fn get_stream_tags(&self, search_stream: SearchStreamTag) -> Result<(u32, u32, Vec<StreamTag>), String> {
            let mut strm_tags: Vec<StreamTag> = vec![StreamTag {
                id: STREAM_TAG_ID,
                name: TAG_NAME.into(),
                count_links: 0,
            }];

            let delta = STREAM_TAG_ID - USER1_ID + 1;
            for stream in self.stream_info_vec.iter() {
                for tag_str in stream.tags.iter() {
                    let tag_name = tag_str.clone();
                    let opt_strm_tag = strm_tags.iter_mut().find(|t| t.name == tag_name);
                    if let Some(strm_tag) = opt_strm_tag {
                        strm_tag.count_links += 1;
                    } else {
                        strm_tags.push(StreamTag {
                            id: stream.user_id + delta,
                            name: tag_name,
                            count_links: 1,
                        });
                    }
                }
            }

            let sort_desc = search_stream.sort_desc.unwrap_or_default();
            strm_tags.sort_by(|a, b| {
                let val1 = if sort_desc { b } else { a };
                let val2 = if sort_desc { a } else { b };
                let result = match search_stream.sort_column {
                    Some(StreamTagSortColumn::Name) => val1.name.partial_cmp(&val2.name).unwrap_or(Ordering::Equal),
                    Some(StreamTagSortColumn::CountLinks) => val1.count_links.partial_cmp(&val2.count_links).unwrap_or(Ordering::Equal),
                    _ => val1.id.partial_cmp(&val2.id).unwrap_or(Ordering::Equal),
                };
                result
            });

            let page = search_stream.page.unwrap_or(SEARCH_STREAM_TAGS_PAGE);
            let limit = search_stream.limit.unwrap_or(SEARCH_STREAM_TAGS_LIMIT);
            let min_idx = (page - 1) * limit;
            let max_idx = min_idx + limit;
            let mut idx = 0;
            strm_tags.retain(|_| {
                let res = min_idx <= idx && idx < max_idx;
                idx += 1;
                res
            });

            Ok((limit, page, strm_tags))
        }
    }

    pub struct StreamDbTest {}

    impl StreamDbTest {
        pub fn stream_ids() -> Vec<i32> {
            vec![
                1, // Owner user idx 0 (live: true)  1100 oliver_taylor
                2, // Owner user idx 1 (live: true)  1101 robert_brown
                3, // Owner user idx 2 (live: false) 1102 mary_williams
                4, // Owner user idx 3  blocked      1103 ava_wilson
            ]
        }
        pub fn create_stream(idx: u8, user_id: i32, title: &str, tags: &str, starttime: DateTime<Utc>) -> StreamAndTags {
            let tags1: Vec<String> = tags.split(',').map(|val| val.to_string()).collect();
            let stream = StreamAndTags::new(STREAM_ID + i32::from(idx), user_id, title, starttime, &tags1);
            stream
        }
        pub fn streams(user_idxs: &[usize]) -> Vec<StreamAndTags> {
            let mut stream_info_vec: Vec<StreamAndTags> = Vec::new();
            let user_ids = USER_IDS.clone();
            for (index, user_idx) in user_idxs.iter().enumerate() {
                if let Some(user_id) = user_ids.get(*user_idx) {
                    let title = format!("title_{}_{}", index, *user_idx);
                    let tags = format!("{},{}{}", TAG_NAME, TAG_NAME, *user_idx + 1);
                    let idx = u8::try_from(index).unwrap();
                    let stream_info = Self::create_stream(idx, *user_id, &title, &tags, Utc::now());
                    stream_info_vec.push(stream_info);
                }
            }
            stream_info_vec
        }
        pub fn cfg_config_strm(config_strm: config_strm::ConfigStrm) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_strm = web::Data::new(config_strm);
                config.app_data(web::Data::clone(&data_config_strm));
            }
        }
        pub fn cfg_stream_db(data_s: Vec<StreamAndTags>) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_stream_db = web::Data::new(StreamDbApp::create(&data_s));
                config.app_data(web::Data::clone(&data_stream_db));
            }
        }
    }
}
