import { HttpErrorResponse } from "@angular/common/http";

// ** Uniqueness of "nickname" and "email". **

export interface UniquenessDto {
    uniqueness: boolean;
}

// ** Registering a new user. (request) **

export interface RegistrUserDto {
    nickname: string;
    email: string;
    password: string;
}

// ** User (password) recovery. (request) **

export interface RecoveryUserDto {
    email: string;
}

// ** User login. (request) **

export interface LoginDto {
    // nickname: MIN=3,MAX=64,"^[a-zA-Z]+[\\w]+$"
    // email: MIN=5,MAX=255,"email_type"
    nickname: string;
    // Matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d\W_]{6,64}$/)
    // password: MIN=6,MAX=64,"[a-z]+","[A-Z]+","[\\d]+" 
    password: string;
}

export interface LoginUserProfileDto {
    id: number;
    nickname: string; // max_len: 255
    email: string; // max_len: 255
    role: string; // UserRole ["User","Admin"]
    avatar: string | undefined | null; // Link to user avatar, optional, min_len=2 max_len=255
    descript: string; // optional, default ""
    theme: string; // Default color theme. ["light","dark"], min_len=2 max_len=32
    locale: string; // Default locale. ["default"], min_len=2 max_len=32
    createdAt: string; // DateTime<Utc> "rfc2822z"
    updatedAt: string; // DateTime<Utc> "rfc2822z"
}
export interface UserTokenResponseDto {
    accessToken: string;
    refreshToken: string;
}

/*?export interface Profile {
    id: number;
    nickname: string; // max_len: 255
    email: string; // max_len: 255
    avatar: string | undefined | null; // Link to user avatar, optional, min_len=2 max_len=255
    theme: string; // Default color theme. ["light","dark"], min_len=2 max_len=32
    locale: string; // Default locale. ["default"], min_len=2 max_len=32
}*/

// ** User login. (Response) **

export interface LoginResponseDto {
    userProfileDto: LoginUserProfileDto;
    tokenUserResponseDto: UserTokenResponseDto;
}

// ** User profile. **

export interface UserDto {
    id: number;
    nickname: string; // max_len: 255
    email: string; // max_len: 255
    role: string; // UserRole ["User","Admin"]
    avatar: string | undefined | null; // Link to user avatar, optional, min_len=2 max_len=255
    descript: string;
    theme: string; // Default color theme. ["light","dark"], min_len=2 max_len=32
    locale: string; // Default locale. ["default"], min_len=2 max_len=32
    createdAt: string; // DateTime<Utc> "rfc2822z"
    updatedAt: string; // DateTime<Utc> "rfc2822z"
}

export class UserDtoUtil {
    public static new(value: any): UserDto {
        return {
            id: value["id"],
            nickname: value["nickname"],
            email: value["email"],
            role: value["role"],
            avatar: value["avatar"],
            descript: value["descript"],
            theme: value["theme"],
            locale: value["locale"],
            createdAt: typeof value.createdAt == "string" ? new Date(value["createdAt"]) : value["createdAt"],
            updatedAt: typeof value.updatedAt == "string" ? new Date(value["updatedAt"]) : value["updatedAt"],
        };
    }
    public static create(userDto?: Partial<UserDto>): UserDto {
        return {
            id: (userDto?.id || -1),
            nickname: (userDto?.nickname || ""),
            email: (userDto?.email || ""),
            role: (userDto?.role || ""),
            avatar: (userDto?.avatar || ""),
            descript: (userDto?.descript || ""),
            theme: (userDto?.theme || ""),
            locale: (userDto?.locale || ""),
            createdAt: (userDto?.createdAt || ""),
            updatedAt: (userDto?.updatedAt || ""),
        };
    }
}

// ** User Tokens **

export interface UserTokenResponseDto {
    accessToken: string;
    refreshToken: string;
}

// ** Refresh Token **

export interface UserTokenDto {
    // refreshToken
    token: string;
}

// ** interface TokenUpdate **

export interface TokenUpdate {
    refreshToken(refreshToken: string): Promise<UserTokenResponseDto | HttpErrorResponse>;
}

// ** UserShortDto **

export interface UserShortDto {
    id: number;
    nickname: string; // max_len: 255
    email: string; // max_len: 255
    role: string; // UserRole ["User","Admin"]
    avatar: string | undefined | null; // Link to user avatar, optional, min_len=2 max_len=255
    settings: Object | undefined | null;
}

export class UserShortDtoUtil {
    public static new(value: any): UserShortDto {
        return {
            id: value["id"],
            nickname: value["nickname"],
            email: value["email"],
            role: value["role"],
            avatar: value["avatar"],
            settings: value["settings"],
        };
    }
    public static create(userShortDto?: Partial<UserShortDto>): UserShortDto {
        return {
            id: (userShortDto?.id || -1),
            nickname: (userShortDto?.nickname || ""),
            email: (userShortDto?.email || ""),
            role: (userShortDto?.role || ""),
            avatar: (userShortDto?.avatar || ""),
            settings: userShortDto?.settings,
        };
    }
}

//
