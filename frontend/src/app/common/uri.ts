declare var APP_DOMAIN: string;

const TEMPLATES_PATTERN = /[\w]+:\/\/|:[\w]+/g;

export class Uri {
  static readonly replacements: { [key: string]: string } = {};

  static {
    // Data Initialization
    Uri.replace('appRoot://', (APP_DOMAIN || (window.location.origin + '/')));
    Uri.replace('appApi://', (APP_DOMAIN || (window.location.origin + '/')));
  }

  /**
   * Specifies a pattern and its replacement on the Uri.
   * @description
   * Used to register a new replacement for app uris.
   * Typically the registration of these replacements takes place in one distinct place (i.e. the app main file).
   * Two kinds of patterns are supported, `:foo` (as path parameter) and `foo://` (as path prefix).
   * @param pattern string
   * @param replacement replacement string or function
   */
  public static replace(pattern: string, replacement: string): void {
    Uri.replacements[pattern] = replacement;
  }

  /**
   * Transforms a app uri using the configured replacement rules.
   * @param str string The uri to transform
   * @returns string The replaced string
   */
  public static appUri(str: string): string {
    const replaced = str.replace(TEMPLATES_PATTERN, (template) => {
      const replacement = Uri.replacements[template];
      return replacement !== undefined ? replacement : template;
    });
    return replaced;
  }

  /**
   * Get the value of a parameter by its name.
   * @param parameter string Parameter name (for example `:foo` or` foo://`).
   * @returns string Parameter value.
   */
  public static get(parameter: string): string | null {
    return parameter != null ? Uri.replacements[parameter] : null;
  }
}
