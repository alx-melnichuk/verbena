// ** Login Profile **

export interface LoginProfileDto {
    // nickname: MIN=3,MAX=64,"^[a-zA-Z]+[\\w]+$"
    // email: MIN=5,MAX=255,"email_type"
    nickname: string; 
    // Matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d\W_]{6,64}$/)
    // password: MIN=6,MAX=64,"[a-z]+","[A-Z]+","[\\d]+" 
    password: string;
  }
    
  export interface LoginProfileResponseDto {
    profileDto: ProfileDto;
    profileTokensDto: ProfileTokensDto;
  }
  
// ** UniquenessDto **

export interface UniquenessDto {
  uniqueness: boolean;
}

// ** ProfileDto **

export interface ProfileDto {
  id: number;
  nickname: string;
  email: string;
  role: string; // UserRole ["User","Admin"]
  avatar: string | undefined | null; // Link to user avatar, optional
  descript: string;
  theme: string; // Default color theme. ["light","dark"]
  createdAt: string; // DateTime<Utc> "rfc2822z"
  updatedAt: string; // DateTime<Utc> "rfc2822z"
}

export class ProfileDtoUtil {
  public static new(value: any): ProfileDto {
    return {
      id: value['id'],
      nickname: value['nickname'],
      email: value['email'],
      role: value['role'],
      avatar: value['avatar'],
      descript: value['descript'],
      theme: value['theme'],
      createdAt: typeof value.createdAt == 'string' ? new Date(value['createdAt']) : value['createdAt'],
      updatedAt: typeof value.updatedAt == 'string' ? new Date(value['updatedAt']) : value['updatedAt'],
    };
  }
  public static create(profileDto?: Partial<ProfileDto>): ProfileDto {
    return {
      id: (profileDto?.id || -1),
      nickname: (profileDto?.nickname || ''),
      email: (profileDto?.email || ''),
      role: (profileDto?.role || ''),
      avatar: (profileDto?.avatar || ''),
      descript: (profileDto?.descript || ''),
      theme: (profileDto?.theme || ''),
      createdAt: (profileDto?.createdAt || ''),
      updatedAt: (profileDto?.updatedAt || ''),
    };
  }
}

// ** Profile Tokens **

export interface ProfileTokensDto {
  accessToken: string;
  refreshToken: string;
}
  
// ** **
