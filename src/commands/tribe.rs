// #[command]
// #[aliases("t")]
// #[usage("<tribe name or tag>")]
// async fn tribe(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
//     api::tribalapi::update_api_data(false, ctx).await;
// 
//     let tribe_name = args.rest();
// 
//     let tw = {
//         let data_read = ctx.data.read().await;
//         data_read
//             .get::<TribalWarsState>()
//             .expect("Expected TribalWars in TypeMap.")
//             .clone()
//     };
// 
//     {
//         let game = tw.write().await;
//         let tribe = game.tribe_by_name(tribe_name);
//         let players = game.players_by_tribe(tribe_name);
// 
//         if tribe.is_none() || players.is_none() {
//             msg.channel_id
//                 .say(&ctx.http, "That tribe doesn't exist you fucking idiot")
//                 .await?;
//             return Ok(());
//         }
// 
//         let tribe = tribe.unwrap();
// 
//         msg.channel_id
//             .send_message(&ctx.http, |m| {
//                 m.content("").embed(|e| {
//                     e.title(format!(
//                         "{} [{}] - {}p.",
//                         tribe.name, tribe.tag, tribe.points
//                     ))
//                         .fields(vec![
//                             ("Members", format!("{}", tribe.members).as_str(), true),
//                             (
//                                 "Average",
//                                 format!("{}", tribe.all_points / tribe.members).as_str(),
//                                 true,
//                             ),
//                         ])
//                         .url(format!(
//                             "https://en125.tribalwars.net/game.php?screen=info_ally&id={}",
//                             tribe.id
//                         ))
//                 })
//             })
//             .await?;
//     }
// 
//     Ok(())
// }
// 
// #[command]
// #[aliases("tm")]
// #[usage("<tribe name or tag>")]
// async fn tribe_members(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
//     api::tribalapi::update_api_data(false, ctx).await;
// 
//     let tribe_name = args.rest();
// 
//     let tw = {
//         let data_read = ctx.data.read().await;
//         data_read
//             .get::<TribalWarsState>()
//             .expect("Expected TribalWars in TypeMap.")
//             .clone()
//     };
// 
//     {
//         let game = tw.write().await;
//         let tribe = game.tribe_by_name(tribe_name);
//         let players = game.players_by_tribe(tribe_name);
// 
//         if tribe.is_none() || players.is_none() {
//             msg.channel_id
//                 .say(&ctx.http, "That tribe doesn't exist you fucking idiot")
//                 .await?;
//             return Ok(());
//         }
// 
//         let tribe = tribe.unwrap();
//         let mut players = players.unwrap();
//         players.sort_by(|a, b| b.points.cmp(&a.points));
// 
//         let player_display: String = players
//             .iter()
//             .map(|p| format!("**{}**: {} villages, {}p.\n", p.name, p.villages, p.points))
//             .collect();
// 
//         msg.channel_id
//             .say(
//                 &ctx.http,
//                 format!("Players in {}:\n{}", tribe.name, player_display),
//             )
//             .await?;
//     }
// 
//     Ok(())
// }