import { environment } from 'src/environments/environment';

// maximum file size for upload
export const MAX_FILE_SIZE = (5 * 1024 * 1024); // 5MB
export const IMAGE_VALID_FILE_TYPES = 'png,jpg,jpeg,gif';


// Format date for moment
export const MOMENT_ISO8601 = 'YYYY-MM-DDTHH:mm:ssZ';
export const MOMENT_ISO8601_DATE = 'YYYY-MM-DD';

// Color scheme
export const SCHEME_LIGHT = 'light';
export const SCHEME_DARK = 'dark';

export const COLOR_SCHEME_LIGHT_AZURE_ORANGE = 'light-azure-orange';
export const COLOR_SCHEME_LIGHT_CYAN_ORANGE = 'light-cyan-orange';
export const COLOR_SCHEME_LIGHT_VIOLET_CHARTREUSE = 'light-violet-chartreuse';
export const COLOR_SCHEME_LIGHT_ORANGE_MAGENTA = 'light-orange-magenta';
export const COLOR_SCHEME_LIGHT_MAGENTA_CYAN = 'light-magenta-cyan';

export const COLOR_SCHEME_DARK_AZURE_ORANGE = 'dark-azure-orange';
export const COLOR_SCHEME_DARK_CYAN_MAGENTA = 'dark-cyan-orange';
export const COLOR_SCHEME_DARK_VIOLET_CHARTREUSE = 'dark-violet-chartreuse';
export const COLOR_SCHEME_DARK_ORANGE_MAGENTA = 'dark-orange-magenta';
export const COLOR_SCHEME_DARK_MAGENTA_CYAN = 'dark-magenta-cyan';

export const COLOR_SCHEME_LIST = [
    // 'light-*'
    COLOR_SCHEME_LIGHT_AZURE_ORANGE,
    COLOR_SCHEME_LIGHT_CYAN_ORANGE,
    // COLOR_SCHEME_LIGHT_VIOLET_MAGENTA,
    COLOR_SCHEME_LIGHT_VIOLET_CHARTREUSE,
    COLOR_SCHEME_LIGHT_ORANGE_MAGENTA,
    COLOR_SCHEME_LIGHT_MAGENTA_CYAN,
    // 'dark-*'
    COLOR_SCHEME_DARK_AZURE_ORANGE,
    COLOR_SCHEME_DARK_CYAN_MAGENTA,
    // COLOR_SCHEME_DARK_VIOLET_MAGENTA,
    COLOR_SCHEME_DARK_VIOLET_CHARTREUSE,
    COLOR_SCHEME_DARK_ORANGE_MAGENTA,
    COLOR_SCHEME_DARK_MAGENTA_CYAN,
];

// Locale
export const LOCALE_EN = 'en-US';
export const LOCALE_DE = 'de-DE';
export const LOCALE_UK = 'uk-UA';
export const LOCALE_LIST = [LOCALE_EN, LOCALE_DE, LOCALE_UK];

// Sign of the "production" mode.
export const ENV_IS_PROD = environment.production;