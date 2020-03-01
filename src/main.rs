use mastors::{
    api,
    api::v1::{
        streaming::{
            StreamType,
            EventType,
        },
        statuses,
    },
    methods::Method,
    Connection,
};

fn main()  {
    let conn = Connection::new_with_path(".env").unwrap();
    let stream = api::v1::streaming::get(&conn, StreamType::User).send().unwrap();

    for event in stream {
        if let EventType::Update(status) = event.unwrap() {
            if status.content().contains("つー!") {
                statuses::post(&conn, "かー!").send().unwrap();
            }
        }
    }
}

/*
    let media_ids: Vec<String> = vec!(
        mastors::api::v1::media::post(&conn, "test3.png").send().await?.id(),
        mastors::api::v1::media::post(&conn, "test2.png").send().await?.id(),
    );

    mastors::api::v1::statuses::post_with_media(&conn, "けだま!", media_ids)?
        .sensitive()
        .visibility(mastors::entities::Visibility::Unlisted)
        .spoiler_text("けだまけだま!")
        .send()
        .await?;

    Ok(())
}
*/


//    println!("{:#?}", mastors::api::v1::instance::get(&conn).send().await?);
//    println!("{:#?}", mastors::api::v1::instance::peers::get(&conn).send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::instance::activity::get(&conn).send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::trends::get(&conn).limit(5).send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::statuses::get(&conn, "103948069277005731").unauthorized().send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::statuses::post(&conn, "test", None, None).spoiler_text("spoiler_text: impl Into<String>").send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::statuses::post(&conn, "poll test", None, mastors::api::v1::statuses::Poll::new(vec!["Option1", "Option2", ""], 3600)?).send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::statuses::post(&conn, None, None, None).spoiler_text("spoiler_text: impl Into<String>").send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::media::post(&conn, "test.png").description("descdesc").send().await.unwrap());
//    println!("{:#?}", mastors::api::v2::media::post(&conn, "test.png").description("descdesc").send().await.unwrap());
//    println!("{:#?}", mastors::api::v1::statuses::post_with_media(&conn, "", vec!("".to_owned())).unwrap().send().await.unwrap());

//    Ok(())
//}

