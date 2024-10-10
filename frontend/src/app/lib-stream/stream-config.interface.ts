
export interface StreamConfigDto {
  // Maximum size for logo files.
  logoMaxSize?: number;
  // List of valid input mime types for logo files.
  // ["image/bmp", "image/gif", "image/jpeg", "image/png"]
  logoValidTypes: Array<string>;
  // Logo files will be converted to this MIME type.
  // Valid values: "image/bmp", "image/gif", "image/jpeg", "image/png"
  logoExt?: string;
  // Maximum width of logo image after saving.
  logoMaxWidth?: number;
  // Maximum height of logo image after saving.
  logoMaxHeight?: number;
}
  