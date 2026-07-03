use rand::RngExt;
use serde::Deserialize;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pokemon {
    id: u32,
    name: String,
    types: Vec<String>,
    stats: Vec<Stat>,
    sprite: String, // url need front_default
    base_experience: u32,
    weight: u16,
    height: u16,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    name: String,
    base_stat: u16,
}

#[tauri::command]
pub async fn roll() -> Result<Pokemon, String> {
    let rand_id = rand::rng().random_range(1..1025);
    let url = format!("https://pokeapi.co/api/v2/pokemon/{}", rand_id);

    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;

    let api_pokemon: ApiPokemon = response
        .json::<ApiPokemon>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(api_pokemon.into())
}

#[derive(Deserialize)]
struct ApiNamed {
    name: String,
}

#[derive(Deserialize)]
struct ApiStatSlot {
    base_stat: u16,
    stat: ApiNamed,
}

#[derive(Deserialize)]
struct ApiSprite {
    front_default: Option<String>,
    other: ApiSpriteOther,
}

#[derive(Deserialize)]
struct ApiSpriteOther {
    #[serde(rename = "official-artwork")]
    official_artwork: ApiSpriteOfficialArtWork,
}

#[derive(Deserialize)]
struct ApiSpriteOfficialArtWork {
    front_default: Option<String>,
}

#[derive(Deserialize)]
struct ApiTypeSlot {
    #[serde(rename = "type")]
    api_type: ApiNamed,
}

#[derive(Deserialize)]
struct ApiPokemon {
    id: u32,
    base_experience: u32,
    name: String,
    sprites: ApiSprite,
    stats: Vec<ApiStatSlot>,
    types: Vec<ApiTypeSlot>,
    weight: u16,
    height: u16,
}

impl From<ApiPokemon> for Pokemon {
    fn from(value: ApiPokemon) -> Self {
        Pokemon {
            id: value.id,
            name: value.name,
            types: value.types.into_iter().map(|t| t.api_type.name).collect(),
            stats: value
                .stats
                .into_iter()
                .map(|s| Stat {
                    name: s.stat.name,
                    base_stat: s.base_stat,
                })
                .collect(),
            sprite: value
                .sprites
                .other
                .official_artwork
                .front_default
                .or(value.sprites.front_default)
                .unwrap_or_default(),
            base_experience: value.base_experience,
            weight: value.weight,
            height: value.height,
        }
    }
}
