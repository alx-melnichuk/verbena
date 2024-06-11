// ** Login User **

export interface LoginUserDto {
  // nickname: MIN=3,MAX=64,"^[a-zA-Z]+[\\w]+$"
  // email: MIN=5,MAX=255,"email_type"
  nickname: string; 
  // Matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d\W_]{6,64}$/)
  // password: MIN=6,MAX=64,"[a-z]+","[A-Z]+","[\\d]+" 
  password: string;
}
  
export interface LoginUserResponseDto {
  userDto: UserDto;
  userTokensDto: UserTokensDto;
}
  
// ** Create User **

export interface CreateUserDto {
  nickname: string; // nickname: MIN=3,MAX=64,"^[a-zA-Z]+[\\w]+$"
  email: string; // email: MIN=5,MAX=255,"email_type"
  // Matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d\W_]{6,64}$/)
  // password: MIN=6,MAX=64,"[a-z]+","[A-Z]+","[\\d]+" 
  password: string;
}

// ** Recovery User **

export interface RecoveryUserDto {
  email: string; // email: MIN=5,MAX=255,"email_type"
}
  
// ** User **

export interface UserDto {
  id: number;
  nickname: string;
  email: string;
  password: string;
  role: string; // UserRole ["User","Admin"]
  createdAt: string; // DateTime<Utc> "rfc2822z"
  updatedAt: string; // DateTime<Utc> "rfc2822z"
}

export class UserDtoUtil {
  public static new(value: any): UserDto {
    return {
      id: value['id'],
      nickname: value['nickname'],
      email: value['email'],
      password: value['password'],
      role: value['role'],
      createdAt: typeof value.createdAt == 'string' ? new Date(value['createdAt']) : value['createdAt'],
      updatedAt: typeof value.updatedAt == 'string' ? new Date(value['updatedAt']) : value['updatedAt'],
    };
  }
  public static create(userDto?: Partial<UserDto>): UserDto {
    return {
      id: (userDto?.id || -1),
      nickname: (userDto?.nickname || ''),
      email: (userDto?.email || ''),
      password: (userDto?.password || ''),
      role: (userDto?.role || ''),
      createdAt: (userDto?.createdAt || ''),
      updatedAt: (userDto?.updatedAt || ''),
    };
  }
}

// ** User Tokens **

export interface UserTokensDto {
  accessToken: string;
  refreshToken: string;
}
 
// ** Refresh Token **

export interface TokenUserDto {
  // refreshToken
  token: string;
}

// ** **