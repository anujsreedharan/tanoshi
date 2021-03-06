use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;
use web_sys::HtmlElement;

use super::component::Spinner;
use crate::app::{job, AppRoute};

use tanoshi_lib::manga::{Chapter as ChapterModel, Manga as MangaModel};
use tanoshi_lib::rest::{GetChaptersResponse, GetMangaResponse};

use serde::{Deserialize, Serialize};

use anyhow;
use yew::services::storage::Area;
use yew::services::StorageService;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FavoriteManga {
    pub source: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddFavoritesResponse {
    pub status: String,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub manga_id: i32,
}

pub struct Detail {
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    token: String,
    manga_id: i32,
    manga: MangaModel,
    chapters: Vec<ChapterModel>,
    is_fetching: bool,
    should_fetch: bool,
    worker: Box<dyn Bridge<job::Worker>>,
    title_ref: NodeRef,
    desc_ref: NodeRef,
}

pub enum Msg {
    MangaReady(GetMangaResponse),
    ChapterReady(GetChaptersResponse),
    Refresh,
    FavoriteEvent,
    Favorited(AddFavoritesResponse),
    Unfavorited(AddFavoritesResponse),
    Noop,
}

impl Component for Detail {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let token = {
            if let Ok(token) = storage.restore("token") {
                token
            } else {
                "".to_string()
            }
        };

        let worker_callback = link.callback(|msg| match msg {
            job::Response::MangaFetched(data) => Msg::MangaReady(data),
            job::Response::ChaptersFetched(data) => Msg::ChapterReady(data),
            _ => Msg::Noop,
        });
        let worker = job::Worker::bridge(worker_callback);

        Detail {
            fetch_task: None,
            link,
            token,
            manga_id: props.manga_id,
            manga: MangaModel::default(),
            chapters: vec![],
            is_fetching: true,
            should_fetch: true,
            worker,
            title_ref: NodeRef::default(),
            desc_ref: NodeRef::default(),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.manga_id != props.manga_id {
            self.manga_id = props.manga_id;
            self.should_fetch = true;
            return true;
        }
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.should_fetch {
            self.get_manga_info();
            self.should_fetch = false;
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::MangaReady(data) => {
                self.manga = data.manga;
                self.get_chapters(false);
                self.is_fetching = false;

                if let Some(title) = self.title_ref.cast::<HtmlElement>() {
                    let _ = title.set_inner_html(self.manga.title.as_str());
                }

                if let Some(desc) = self.desc_ref.cast::<HtmlElement>() {
                    let _ = desc.set_inner_html(self.manga.description.as_ref().unwrap_or(&"N/A".to_string()));
                }

                return false;
            }
            Msg::ChapterReady(data) => {
                self.chapters = data.chapters;
                self.is_fetching = false;
            }
            Msg::Refresh => {
                self.get_chapters(true);
            }
            Msg::FavoriteEvent => {
                if self.manga.is_favorite {
                    self.unfavorite();
                } else {
                    self.favorite();
                }
            }
            Msg::Favorited(data) => {
                if data.status == "success" {
                    self.manga.is_favorite = true;
                }
            }
            Msg::Unfavorited(data) => {
                if data.status == "success" {
                    self.manga.is_favorite = false;
                }
            }
            Msg::Noop => {
                return false;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="container pb-20" style="margin-top: calc(env(safe-area-inset-top) + 3rem)">
            <div class="w-full px-2 pb-2 flex justify-between block fixed inset-x-0 top-0 z-50 bg-tachiyomi-blue shadow" style="padding-top: calc(env(safe-area-inset-top) + .5rem)">
                <button
                onclick=self.link.callback(|_| Msg::FavoriteEvent)
                class="hover:bg-tachiyomi-blue-darker rounded flex-none">
                    <svg class="mx-2 my-auto self-center" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" >
                    {
                        if !self.manga.is_favorite {
                            html!{<path class="heroicon-ui" d="M6.1 21.98a1 1 0 0 1-1.45-1.06l1.03-6.03-4.38-4.26a1 1 0 0 1 .56-1.71l6.05-.88 2.7-5.48a1 1 0 0 1 1.8 0l2.7 5.48 6.06.88a1 1 0 0 1 .55 1.7l-4.38 4.27 1.04 6.03a1 1 0 0 1-1.46 1.06l-5.4-2.85-5.42 2.85zm4.95-4.87a1 1 0 0 1 .93 0l4.08 2.15-.78-4.55a1 1 0 0 1 .29-.88l3.3-3.22-4.56-.67a1 1 0 0 1-.76-.54l-2.04-4.14L9.47 9.4a1 1 0 0 1-.75.54l-4.57.67 3.3 3.22a1 1 0 0 1 .3.88l-.79 4.55 4.09-2.15z"/>}
                        } else {
                            html!{<path class="heroicon-ui" d="m6.1,21.98a1,1 0 0 1 -1.45,-1.059999l1.03,-6.03l-4.38,-4.26a1,1 0 0 1 0.56,-1.71l6.05,-0.88l2.7,-5.48a1,1 0 0 1 1.8,0l2.7,5.48l6.06,0.88a1,1 0 0 1 0.549999,1.7l-4.379999,4.27l1.039999,6.03a1,1 0 0 1 -1.459999,1.059999l-5.4,-2.85l-5.420001,2.85z"/>}
                        }
                    }
                    </svg>
                </button>
                <RouterAnchor<AppRoute>
                classes="hover:bg-tachiyomi-blue-darker rounded flex-grow text-white text-center my-1 mx-2 px-3 w-full"
                route=AppRoute::Chapter(self.manga.last_read.unwrap_or(self.chapters.last().unwrap_or(&ChapterModel::default()).id), (self.manga.last_page.as_ref().unwrap_or(&0) + 1) as usize)>
                    //<svg class="fill-current mx-2 my-auto self-center" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path class="heroicon-ui" d="M7 5H5v14h14V5h-2v10a1 1 0 0 1-1.45.9L12 14.11l-3.55 1.77A1 1 0 0 1 7 15V5zM5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2zm4 2v8.38l2.55-1.27a1 1 0 0 1 .9 0L15 13.38V5H9z"/></svg>
                    {"Read"}
                </RouterAnchor<AppRoute>>
                <button
                onclick=self.link.callback(|_| Msg::Refresh)
                class="hover:bg-tachiyomi-blue-darker rounded flex-none">
                    <svg class="mx-2 my-auto self-center" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" >
                        <path class="heroicon-ui" d="M6 18.7V21a1 1 0 0 1-2 0v-5a1 1 0 0 1 1-1h5a1 1 0 1 1 0 2H7.1A7 7 0 0 0 19 12a1 1 0 1 1 2 0 9 9 0 0 1-15 6.7zM18 5.3V3a1 1 0 0 1 2 0v5a1 1 0 0 1-1 1h-5a1 1 0 0 1 0-2h2.9A7 7 0 0 0 5 12a1 1 0 1 1-2 0 9 9 0 0 1 15-6.7z"/>
                    </svg>
                </button>
            </div>
            <Spinner is_active={self.is_fetching} is_fullscreen=true />
            <div class="m-2 flex md:flex-row sm:flex-col">
                <div class="flex-shrink-0 lg:m-2 sm:mx-auto sm:my-2">
                    <div class="relative my-4">
                        <img class="manga-cover sm:cover cover-sm" src=self.manga.thumbnail_url />
                    </div>
                </div>
                <div class="flex flex-col m-2">
                    <span ref=self.title_ref.clone() class="md:text-xl sm:text-base font-bold"></span>
                    <span class="md:text-xl sm:text-sm font-semibold">{self.manga.author.join(", ").to_owned()}</span>
                    <span class="md:text-xl sm:text-sm font-semibold">{self.manga.status.as_ref().unwrap_or(&"N/A".to_string()).to_owned()}</span>
                    //<p class="md:text-xl sm:text-sm font-medium break-normal">{self.manga.genre.join(", ").to_owned()}</p>
                    <p ref=self.desc_ref.clone() class="break-normal md:text-base sm:text-xs"></p>
                </div>
            </div>
            <div class="w-6/7 mx-2 flex flex-col rounded-lg">
                {
                    for self.chapters.iter().map(|(chapter)| html!{
                        <div class={
                            format!("hover:bg-gray-200 border border-grey-light {}", if chapter.read.unwrap_or(0) > 0 {"bg-gray-200"} else {""})
                        }>
                            <RouterAnchor<AppRoute>
                            classes="inline-flex flex-wrap justify-between w-full px-2 py-2 text-left block"
                            route=AppRoute::Chapter(chapter.id, (chapter.read.unwrap_or(0) + 1) as usize)>
                                <span class="px-2 py-2">
                                {
                                    format!("{}{}{}",
                                    if let Some(v) = &chapter.vol {
                                        format!("Vol. {} ", v)
                                    } else {
                                        "".to_string()
                                    },
                                    if let Some(c) = &chapter.no {
                                        format!("Ch. {} ", c)
                                    } else {
                                        "".to_string()
                                    },
                                    {
                                    let t = chapter.title.as_ref().unwrap();
                                    if !t.is_empty() {
                                        format!("{}", t)
                                    } else {
                                        "".to_string()
                                    }
                                    })
                                }
                                </span>
                                <span class="flex-shrink-0 text-sm text-gray-800 px-2 py-2">{chapter.uploaded.date()}</span>
                            </RouterAnchor<AppRoute>>
                        </div>
                    })
                }
            </div>
            </div>
        }
    }
}

impl Detail {
    fn get_manga_info(&mut self) {
        self.worker.send(job::Request::FetchManga(self.manga_id));
        self.is_fetching = true;
    }

    fn get_chapters(&mut self, refresh: bool) {
        self.worker
            .send(job::Request::FetchChapters(self.manga_id, refresh));
        self.is_fetching = true;
    }

    fn favorite(&mut self) {
        let req = Request::post(format!("/api/favorites/manga/{}", self.manga_id))
            .header("Authorization", self.token.to_owned())
            .header("Content-Type", "application/json")
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<AddFavoritesResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::Favorited(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn unfavorite(&mut self) {
        let req = Request::delete(format!("/api/favorites/manga/{}", self.manga_id))
            .header("Authorization", self.token.to_owned())
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<AddFavoritesResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::Unfavorited(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }
}
