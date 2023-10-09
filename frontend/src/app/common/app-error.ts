export interface AppError {
  errCode: string;
  errMsg: string;
  params: {
    [key: string]: string | number | null;
  };
}
