export interface UserDto {
  id: string; // NotEmpty, ID
  email: string; // 255 NotEmpty, Email
  nickname: string; // 64 NotEmpty
  avatar: string;
  description: string;
}

export interface SendUserRegistrationDto {
  email: string;
  nickname: string; // Matches(/^[a-zA-Z0-9]+$/i)
  password: string; // Matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d\W_]{6,}$/)
  locale?: string; // = 'default';
}

export interface LoginUserDto {
  nickname: string;
  password: string;
}

export interface UserTokensDto {
  accessToken: string;
  refreshToken: string;
}

export interface LoginUserResponseDto {
  userDto: UserDto;
  userTokensDto: UserTokensDto;
}
