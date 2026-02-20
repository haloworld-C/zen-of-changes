use eframe::egui;
use rand::Rng;
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Yao {
    Yin,
    Yang,
}

#[derive(Clone, Copy, Debug)]
struct Trigram {
    index: u8,
    name: &'static str,
    symbol: &'static str,
    lines: [Yao; 3], // 从下到上
}

#[derive(Clone, Debug)]
struct HexagramData {
    name: String,
    symbol: String,
    gua_ci: String,
    yao_ci: [String; 6], // 初爻到上爻
    yi_zhuan: String,
    xi_ci_zhuan: String,
    xiang_zhuan: String,
}

#[derive(Clone, Debug)]
struct DivinationResult {
    lower: Trigram,
    upper: Trigram,
    moving_line: u8,
    lines: [Yao; 6], // 从下到上
    hexagram: HexagramData,
}

struct AppState {
    result: Option<DivinationResult>,
    table: HashMap<(u8, u8), HexagramData>, // key: (upper, lower)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "The Zen of Changes",
        options,
        Box::new(|cc| {
            configure_chinese_font(&cc.egui_ctx);
            Ok(Box::new(AppState {
                result: None,
                table: build_hexagram_table(),
            }))
        }),
    )
}

fn configure_chinese_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some((font_name, font_bytes)) = load_first_available_cjk_font() {
        fonts
            .font_data
            .insert(font_name.clone(), egui::FontData::from_owned(font_bytes).into());

        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, font_name.clone());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.insert(0, font_name);
        }
    }

    ctx.set_fonts(fonts);
}

fn load_first_available_cjk_font() -> Option<(String, Vec<u8>)> {
    let candidates = [
        (
            "NotoSansCJK",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ),
        (
            "NotoSansSC",
            "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
        ),
        ("WenQuanYi", "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc"),
        ("SimHei", "C:\\Windows\\Fonts\\simhei.ttf"),
        ("MicrosoftYaHei", "C:\\Windows\\Fonts\\msyh.ttc"),
        ("PingFangSC", "/System/Library/Fonts/PingFang.ttc"),
    ];

    for (name, path) in candidates {
        if let Ok(bytes) = fs::read(path) {
            return Some((name.to_owned(), bytes));
        }
    }

    None
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("易经随机卦象演示");
            ui.label("随机生成三组数字：下爻卦(1-8)、上爻卦(1-8)、主爻(1-6)");
            ui.add_space(8.0);

            if ui.button("生成随机卦").clicked() {
                self.result = Some(generate_result(&self.table));
            }

            ui.add_space(12.0);

            if let Some(result) = &self.result {
                ui.group(|ui| {
                    ui.label(format!(
                        "随机数：下爻={}，上爻={}，主爻={}",
                        result.lower.index, result.upper.index, result.moving_line
                    ));
                    ui.label(format!(
                        "下卦：{} {}    上卦：{} {}",
                        result.lower.symbol,
                        result.lower.name,
                        result.upper.symbol,
                        result.upper.name
                    ));
                    ui.label(format!(
                        "本卦：{} {}",
                        result.hexagram.symbol, result.hexagram.name
                    ));
                });

                ui.add_space(8.0);
                ui.label("卦象（上到下）：");
                render_hexagram_lines(ui, result);

                ui.add_space(8.0);
                ui.separator();
                ui.label(format!("卦辞：{}", result.hexagram.gua_ci));
                ui.label(format!(
                    "主爻爻辞（第{}爻）：{}",
                    result.moving_line,
                    result.hexagram.yao_ci[(result.moving_line - 1) as usize]
                ));
                ui.label(format!("易传：{}", result.hexagram.yi_zhuan));
                ui.label(format!("系辞传：{}", result.hexagram.xi_ci_zhuan));
                ui.label(format!("象传：{}", result.hexagram.xiang_zhuan));
            } else {
                ui.label("点击“生成随机卦”开始。");
            }
        });
    }
}

fn render_hexagram_lines(ui: &mut egui::Ui, result: &DivinationResult) {
    for line_no in (1..=6).rev() {
        let idx = (line_no - 1) as usize;
        let line = match result.lines[idx] {
            Yao::Yang => "────────",
            Yao::Yin => "────  ────",
        };
        if line_no == result.moving_line {
            ui.colored_label(egui::Color32::LIGHT_RED, format!("{}  ← 主爻", line));
        } else {
            ui.label(line);
        }
    }
}

fn generate_result(table: &HashMap<(u8, u8), HexagramData>) -> DivinationResult {
    let mut rng = rand::thread_rng();
    let lower_idx = rng.gen_range(1..=8);
    let upper_idx = rng.gen_range(1..=8);
    let moving_line = rng.gen_range(1..=6);

    let lower = trigram_by_index(lower_idx);
    let upper = trigram_by_index(upper_idx);

    let mut lines = [Yao::Yin; 6];
    lines[0] = lower.lines[0];
    lines[1] = lower.lines[1];
    lines[2] = lower.lines[2];
    lines[3] = upper.lines[0];
    lines[4] = upper.lines[1];
    lines[5] = upper.lines[2];

    let hexagram = table
        .get(&(upper_idx, lower_idx))
        .cloned()
        .unwrap_or_else(|| fallback_hexagram(lower, upper));

    DivinationResult {
        lower,
        upper,
        moving_line,
        lines,
        hexagram,
    }
}

fn trigram_by_index(i: u8) -> Trigram {
    match i {
        1 => Trigram {
            index: 1,
            name: "乾",
            symbol: "☰",
            lines: [Yao::Yang, Yao::Yang, Yao::Yang],
        },
        2 => Trigram {
            index: 2,
            name: "兑",
            symbol: "☱",
            lines: [Yao::Yang, Yao::Yang, Yao::Yin],
        },
        3 => Trigram {
            index: 3,
            name: "离",
            symbol: "☲",
            lines: [Yao::Yang, Yao::Yin, Yao::Yang],
        },
        4 => Trigram {
            index: 4,
            name: "震",
            symbol: "☳",
            lines: [Yao::Yang, Yao::Yin, Yao::Yin],
        },
        5 => Trigram {
            index: 5,
            name: "巽",
            symbol: "☴",
            lines: [Yao::Yin, Yao::Yang, Yao::Yang],
        },
        6 => Trigram {
            index: 6,
            name: "坎",
            symbol: "☵",
            lines: [Yao::Yin, Yao::Yang, Yao::Yin],
        },
        7 => Trigram {
            index: 7,
            name: "艮",
            symbol: "☶",
            lines: [Yao::Yin, Yao::Yin, Yao::Yang],
        },
        _ => Trigram {
            index: 8,
            name: "坤",
            symbol: "☷",
            lines: [Yao::Yin, Yao::Yin, Yao::Yin],
        },
    }
}

fn fallback_hexagram(lower: Trigram, upper: Trigram) -> HexagramData {
    let name = format!("上{}下{}", upper.name, lower.name);
    let symbol = format!("{}{}", upper.symbol, lower.symbol);
    HexagramData {
        name,
        symbol,
        gua_ci: "该卦卦辞待补全。".to_owned(),
        yao_ci: [
            "初爻爻辞待补全。".to_owned(),
            "二爻爻辞待补全。".to_owned(),
            "三爻爻辞待补全。".to_owned(),
            "四爻爻辞待补全。".to_owned(),
            "五爻爻辞待补全。".to_owned(),
            "上爻爻辞待补全。".to_owned(),
        ],
        yi_zhuan: "易传内容待补全。".to_owned(),
        xi_ci_zhuan: "系辞传内容待补全。".to_owned(),
        xiang_zhuan: "象传内容待补全。".to_owned(),
    }
}

fn build_hexagram_table() -> HashMap<(u8, u8), HexagramData> {
    let mut map = HashMap::new();

    map.insert(
        (1, 1),
        HexagramData {
            name: "乾".to_owned(),
            symbol: "䷀".to_owned(),
            gua_ci: "元亨利贞。".to_owned(),
            yao_ci: [
                "初九：潜龙勿用。".to_owned(),
                "九二：见龙在田，利见大人。".to_owned(),
                "九三：君子终日乾乾，夕惕若，厉无咎。".to_owned(),
                "九四：或跃在渊，无咎。".to_owned(),
                "九五：飞龙在天，利见大人。".to_owned(),
                "上九：亢龙有悔。".to_owned(),
            ],
            yi_zhuan: "《彖》：大哉乾元，万物资始，乃统天。".to_owned(),
            xi_ci_zhuan: "《系辞》：乾以易知。".to_owned(),
            xiang_zhuan: "《象》：天行健，君子以自强不息。".to_owned(),
        },
    );

    map.insert(
        (8, 8),
        HexagramData {
            name: "坤".to_owned(),
            symbol: "䷁".to_owned(),
            gua_ci: "元亨，利牝马之贞。君子有攸往，先迷后得主，利。西南得朋，东北丧朋。安贞吉。"
                .to_owned(),
            yao_ci: [
                "初六：履霜，坚冰至。".to_owned(),
                "六二：直方大，不习无不利。".to_owned(),
                "六三：含章可贞。或从王事，无成有终。".to_owned(),
                "六四：括囊；无咎，无誉。".to_owned(),
                "六五：黄裳，元吉。".to_owned(),
                "上六：龙战于野，其血玄黄。".to_owned(),
            ],
            yi_zhuan: "《彖》：至哉坤元，万物资生，乃顺承天。".to_owned(),
            xi_ci_zhuan: "《系辞》：坤以简能。".to_owned(),
            xiang_zhuan: "《象》：地势坤，君子以厚德载物。".to_owned(),
        },
    );

    map.insert(
        (6, 6),
        HexagramData {
            name: "坎".to_owned(),
            symbol: "䷜".to_owned(),
            gua_ci: "习坎，有孚，维心亨，行有尚。".to_owned(),
            yao_ci: [
                "初六：习坎，入于坎窞，凶。".to_owned(),
                "九二：坎有险，求小得。".to_owned(),
                "六三：来之坎坎，险且枕，入于坎窞，勿用。".to_owned(),
                "六四：樽酒簋贰，用缶，纳约自牖，终无咎。".to_owned(),
                "九五：坎不盈，祗既平，无咎。".to_owned(),
                "上六：系用徽纆，寘于丛棘，三岁不得，凶。".to_owned(),
            ],
            yi_zhuan: "《彖》：习坎，重险也。".to_owned(),
            xi_ci_zhuan: "《系辞》：坎，陷也。".to_owned(),
            xiang_zhuan: "《象》：水洊至，习坎。君子以常德行，习教事。".to_owned(),
        },
    );

    map.insert(
        (3, 3),
        HexagramData {
            name: "离".to_owned(),
            symbol: "䷝".to_owned(),
            gua_ci: "利贞，亨。畜牝牛，吉。".to_owned(),
            yao_ci: [
                "初九：履错然，敬之无咎。".to_owned(),
                "六二：黄离，元吉。".to_owned(),
                "九三：日昃之离，不鼓缶而歌，则大耋之嗟，凶。".to_owned(),
                "九四：突如其来如，焚如，死如，弃如。".to_owned(),
                "六五：出涕沱若，戚嗟若，吉。".to_owned(),
                "上九：王用出征，有嘉折首，获匪其丑，无咎。".to_owned(),
            ],
            yi_zhuan: "《彖》：离，丽也；日月丽乎天，百谷草木丽乎土。".to_owned(),
            xi_ci_zhuan: "《系辞》：离，附也。".to_owned(),
            xiang_zhuan: "《象》：明两作，离。大人以继明照于四方。".to_owned(),
        },
    );

    map
}
