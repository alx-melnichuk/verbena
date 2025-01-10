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

export const COLOR_SCHEME_LIGHT_AZURE_BLUE = 'light-azure_blue';
export const COLOR_SCHEME_LIGHT_ROSE_RED = 'light-rose_red';
export const COLOR_SCHEME_LIGHT_CYAN_ORANGE = 'light-cyan_orange';

export const COLOR_SCHEME_DARK_AZURE_BLUE = 'dark-azure_blue';
export const COLOR_SCHEME_DARK_ROSE_RED = 'dark-rose_red';
export const COLOR_SCHEME_DARK_CYAN_ORANGE = 'dark-cyan_orange';

export const COLOR_SCHEME_LIST = [
  COLOR_SCHEME_LIGHT_AZURE_BLUE, COLOR_SCHEME_LIGHT_ROSE_RED, COLOR_SCHEME_LIGHT_CYAN_ORANGE,
  COLOR_SCHEME_DARK_AZURE_BLUE , COLOR_SCHEME_DARK_ROSE_RED , COLOR_SCHEME_DARK_CYAN_ORANGE,
];

// Locale
export const LOCALE_EN = 'en-US';
export const LOCALE_DE = 'de-DE';
export const LOCALE_UK = 'uk-UA';
export const LOCALE_LIST = [LOCALE_EN, LOCALE_DE, LOCALE_UK];

// Sign of the "production" mode.
export const ENV_IS_PROD = environment.production;