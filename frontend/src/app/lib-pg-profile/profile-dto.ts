// ** ModifyProfileDto **

export interface ProfileSettings {
    param1?: number | undefined;
    param2?: string | undefined;
}

export interface ModifyProfileDto {
    nickname?: string | undefined;
    email?: string | undefined;
    role?: string; // UserRole ["User","Admin"]
    descript?: string | undefined;
    theme?: string | undefined; // Default color theme. ["light","dark"]
    locale?: string | undefined; // Default locale. ["default"]
}

// ** NewPasswordProfileDto **

export interface NewPasswordProfileDto {
    password: string;
    newPassword: string;
}

// ** **
