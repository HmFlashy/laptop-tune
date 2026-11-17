use std::env;

pub struct Strings {
    pub close: &'static str,
    pub refresh: &'static str,
    pub power_profile: &'static str,
    pub reset_default: &'static str,
    pub start_on_login: &'static str,
    pub auto_switch: &'static str,
    pub on_battery: &'static str,
    pub on_ac: &'static str,
    pub battery: &'static str,
    pub no_battery: &'static str,
    pub charge_thresholds: &'static str,
    pub gpus_and_sleep: &'static str,
    pub thermals: &'static str,
    pub show_hide: &'static str,
    pub quit: &'static str,
    pub defaults_restored: &'static str,
    pub action_failed: &'static str,
}

const EN: Strings = Strings {
    close: "Close",
    refresh: "Refresh",
    power_profile: "Power Profile",
    reset_default: "Reset Default",
    start_on_login: "Start on login",
    auto_switch: "Auto-switch:",
    on_battery: "On Battery:",
    on_ac: "On AC:",
    battery: "Battery",
    no_battery: "No battery detected.",
    charge_thresholds: "Charge thresholds",
    gpus_and_sleep: "GPUs & Sleep",
    thermals: "Thermals",
    show_hide: "Show / Hide",
    quit: "Quit",
    defaults_restored: "Defaults restored (balanced).",
    action_failed: "Action failed or cancelled.",
};

const FR: Strings = Strings {
    close: "Fermer",
    refresh: "Actualiser",
    power_profile: "Profil d'alimentation",
    reset_default: "Par défaut",
    start_on_login: "Lancer au démarrage",
    auto_switch: "Bascule auto :",
    on_battery: "Sur batterie :",
    on_ac: "Sur secteur :",
    battery: "Batterie",
    no_battery: "Aucune batterie détectée.",
    charge_thresholds: "Seuils de charge",
    gpus_and_sleep: "GPU & Veille",
    thermals: "Températures",
    show_hide: "Afficher / Masquer",
    quit: "Quitter",
    defaults_restored: "Valeurs par défaut restaurées (balanced).",
    action_failed: "Action annulée ou échouée.",
};

const ES: Strings = Strings {
    close: "Cerrar",
    refresh: "Actualizar",
    power_profile: "Perfil de energía",
    reset_default: "Restablecer",
    start_on_login: "Iniciar con el sistema",
    auto_switch: "Cambio auto:",
    on_battery: "En batería:",
    on_ac: "En corriente:",
    battery: "Batería",
    no_battery: "No se detectó batería.",
    charge_thresholds: "Umbrales de carga",
    gpus_and_sleep: "GPU y suspensión",
    thermals: "Temperaturas",
    show_hide: "Mostrar / Ocultar",
    quit: "Salir",
    defaults_restored: "Valores predeterminados restaurados.",
    action_failed: "Acción cancelada o fallida.",
};

const DE: Strings = Strings {
    close: "Schließen",
    refresh: "Aktualisieren",
    power_profile: "Energieprofil",
    reset_default: "Standard",
    start_on_login: "Beim Anmelden starten",
    auto_switch: "Auto-Wechsel:",
    on_battery: "Akkubetrieb:",
    on_ac: "Netzbetrieb:",
    battery: "Akku",
    no_battery: "Kein Akku erkannt.",
    charge_thresholds: "Ladeschwellen",
    gpus_and_sleep: "GPU & Standby",
    thermals: "Temperaturen",
    show_hide: "Ein- / Ausblenden",
    quit: "Beenden",
    defaults_restored: "Standardwerte wiederhergestellt.",
    action_failed: "Aktion abgebrochen oder fehlgeschlagen.",
};

const IT: Strings = Strings {
    close: "Chiudi",
    refresh: "Aggiorna",
    power_profile: "Profilo energetico",
    reset_default: "Predefinito",
    start_on_login: "Avvia all'accesso",
    auto_switch: "Cambio auto:",
    on_battery: "A batteria:",
    on_ac: "Alimentazione:",
    battery: "Batteria",
    no_battery: "Nessuna batteria rilevata.",
    charge_thresholds: "Soglie di carica",
    gpus_and_sleep: "GPU e sospensione",
    thermals: "Temperature",
    show_hide: "Mostra / Nascondi",
    quit: "Esci",
    defaults_restored: "Impostazioni predefinite ripristinate.",
    action_failed: "Azione annullata o non riuscita.",
};

const PT: Strings = Strings {
    close: "Fechar",
    refresh: "Atualizar",
    power_profile: "Perfil de energia",
    reset_default: "Padrão",
    start_on_login: "Iniciar na sessão",
    auto_switch: "Troca auto:",
    on_battery: "Na bateria:",
    on_ac: "Na tomada:",
    battery: "Bateria",
    no_battery: "Nenhuma bateria detetada.",
    charge_thresholds: "Limites de carga",
    gpus_and_sleep: "GPU e suspensão",
    thermals: "Temperaturas",
    show_hide: "Mostrar / Ocultar",
    quit: "Sair",
    defaults_restored: "Padrões restaurados.",
    action_failed: "Ação cancelada ou falhou.",
};

const NL: Strings = Strings {
    close: "Sluiten",
    refresh: "Vernieuwen",
    power_profile: "Energieprofiel",
    reset_default: "Standaard",
    start_on_login: "Starten bij aanmelden",
    auto_switch: "Auto-wissel:",
    on_battery: "Op accu:",
    on_ac: "Op netstroom:",
    battery: "Accu",
    no_battery: "Geen accu gedetecteerd.",
    charge_thresholds: "Laaddrempels",
    gpus_and_sleep: "GPU & Slaapstand",
    thermals: "Temperaturen",
    show_hide: "Tonen / Verbergen",
    quit: "Afsluiten",
    defaults_restored: "Standaardwaarden hersteld.",
    action_failed: "Actie geannuleerd of mislukt.",
};

const PL: Strings = Strings {
    close: "Zamknij",
    refresh: "Odśwież",
    power_profile: "Profil zasilania",
    reset_default: "Domyślne",
    start_on_login: "Uruchom przy logowaniu",
    auto_switch: "Auto-zmiana:",
    on_battery: "Na baterii:",
    on_ac: "Na zasilaczu:",
    battery: "Bateria",
    no_battery: "Nie wykryto baterii.",
    charge_thresholds: "Progi ładowania",
    gpus_and_sleep: "GPU i uśpienie",
    thermals: "Temperatury",
    show_hide: "Pokaż / Ukryj",
    quit: "Zakończ",
    defaults_restored: "Przywrócono ustawienia domyślne.",
    action_failed: "Akcja anulowana lub nieudana.",
};

const TR: Strings = Strings {
    close: "Kapat",
    refresh: "Yenile",
    power_profile: "Güç Profili",
    reset_default: "Varsayılan",
    start_on_login: "Oturumda başlat",
    auto_switch: "Oto-geçiş:",
    on_battery: "Pilde:",
    on_ac: "Prize takılı:",
    battery: "Pil",
    no_battery: "Pil algılanmadı.",
    charge_thresholds: "Şarj eşikleri",
    gpus_and_sleep: "GPU ve Uyku",
    thermals: "Sıcaklıklar",
    show_hide: "Göster / Gizle",
    quit: "Çıkış",
    defaults_restored: "Varsayılanlar geri yüklendi.",
    action_failed: "İşlem iptal edildi veya başarısız.",
};

const RU: Strings = Strings {
    close: "Закрыть",
    refresh: "Обновить",
    power_profile: "Профиль питания",
    reset_default: "По умолчанию",
    start_on_login: "Запускать при входе",
    auto_switch: "Автопереключение:",
    on_battery: "От батареи:",
    on_ac: "От сети:",
    battery: "Батарея",
    no_battery: "Батарея не обнаружена.",
    charge_thresholds: "Пороги заряда",
    gpus_and_sleep: "GPU и сон",
    thermals: "Температуры",
    show_hide: "Показать / Скрыть",
    quit: "Выход",
    defaults_restored: "Настройки по умолчанию восстановлены.",
    action_failed: "Действие отменено или не удалось.",
};

const UK: Strings = Strings {
    close: "Закрити",
    refresh: "Оновити",
    power_profile: "Профіль живлення",
    reset_default: "За замовчуванням",
    start_on_login: "Запускати при вході",
    auto_switch: "Автоперемикання:",
    on_battery: "Від батареї:",
    on_ac: "Від мережі:",
    battery: "Батарея",
    no_battery: "Батарею не знайдено.",
    charge_thresholds: "Пороги заряду",
    gpus_and_sleep: "GPU та сон",
    thermals: "Температури",
    show_hide: "Показати / Сховати",
    quit: "Вихід",
    defaults_restored: "Стандартні налаштування відновлено.",
    action_failed: "Дію скасовано або помилка.",
};

const ZH: Strings = Strings {
    close: "关闭",
    refresh: "刷新",
    power_profile: "电源模式",
    reset_default: "恢复默认",
    start_on_login: "开机自启",
    auto_switch: "自动切换:",
    on_battery: "电池:",
    on_ac: "电源:",
    battery: "电池",
    no_battery: "未检测到电池。",
    charge_thresholds: "充电阈值",
    gpus_and_sleep: "显卡与休眠",
    thermals: "温度",
    show_hide: "显示 / 隐藏",
    quit: "退出",
    defaults_restored: "已恢复默认设置 (balanced)。",
    action_failed: "操作已取消或失败。",
};

const JA: Strings = Strings {
    close: "閉じる",
    refresh: "更新",
    power_profile: "電源プロファイル",
    reset_default: "デフォルトに戻す",
    start_on_login: "ログイン時に起動",
    auto_switch: "自動切替:",
    on_battery: "バッテリー:",
    on_ac: "電源接続:",
    battery: "バッテリー",
    no_battery: "バッテリーが検出されません。",
    charge_thresholds: "充電しきい値",
    gpus_and_sleep: "GPU・スリープ",
    thermals: "温度",
    show_hide: "表示 / 非表示",
    quit: "終了",
    defaults_restored: "デフォルト設定に戻しました。",
    action_failed: "操作がキャンセルまたは失敗しました。",
};

const KO: Strings = Strings {
    close: "닫기",
    refresh: "새로고침",
    power_profile: "전원 프로필",
    reset_default: "기본값 복원",
    start_on_login: "로그인 시 시작",
    auto_switch: "자동 전환:",
    on_battery: "배터리:",
    on_ac: "전원 연결:",
    battery: "배터리",
    no_battery: "배터리가 감지되지 않았습니다.",
    charge_thresholds: "충전 임계값",
    gpus_and_sleep: "GPU 및 절전",
    thermals: "온도",
    show_hide: "표시 / 숨기기",
    quit: "종료",
    defaults_restored: "기본값으로 복원되었습니다.",
    action_failed: "작업이 취소되었거나 실패했습니다.",
};

pub fn strings_for_locale(locale: &str) -> &'static Strings {
    let code = locale
        .split(['_', '.', '-', '@'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase();
    match code.as_str() {
        "fr" => &FR,
        "es" => &ES,
        "de" => &DE,
        "it" => &IT,
        "pt" => &PT,
        "nl" => &NL,
        "pl" => &PL,
        "tr" => &TR,
        "ru" => &RU,
        "uk" => &UK,
        "zh" => &ZH,
        "ja" => &JA,
        "ko" => &KO,
        _ => &EN,
    }
}

pub fn tr() -> &'static Strings {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = env::var(var) {
            let trimmed = val.trim();
            if !trimmed.is_empty() && trimmed != "C" && trimmed != "POSIX" {
                return strings_for_locale(trimmed);
            }
        }
    }
    &EN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_resolution() {
        assert_eq!(strings_for_locale("fr_FR.UTF-8").close, "Fermer");
        assert_eq!(strings_for_locale("de_DE").close, "Schließen");
        assert_eq!(strings_for_locale("es-ES").close, "Cerrar");
        assert_eq!(strings_for_locale("ja_JP.UTF-8").close, "閉じる");
        assert_eq!(strings_for_locale("C").close, "Close");
        assert_eq!(strings_for_locale("en_US.UTF-8").close, "Close");
    }
}
