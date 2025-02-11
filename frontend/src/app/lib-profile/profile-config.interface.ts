
export interface ProfileConfigDto {
    // Maximum size for avatar files.
    avatarMaxSize?: number;
    // List of valid input mime types for avatar files.
    // ["image/bmp", "image/gif", "image/jpeg", "image/png"]
    avatarValidTypes: Array<string>;
    // Avatar files will be converted to this MIME type.
    // Valid values: "image/bmp", "image/gif", "image/jpeg", "image/png"
    avatarExt?: string;
    // Maximum width of avatar image after saving.
    avatarMaxWidth?: number;
    // Maximum height of avatar image after saving.
    avatarMaxHeight?: number;
}
