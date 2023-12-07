export enum AlertMode {
  comment = 'comment',
  info = 'info',
  warning = 'warning',
  error = 'error',
  success = 'success',
}

export interface AlertInterface {
  isOneLine?: boolean;
  closeable?: boolean;
  mode?: AlertMode;
  title?: string;
  message: string;
  messageHtml?: string;
}

export const AlertDurationByMode = {
  [AlertMode.comment]: 4000,
  [AlertMode.info]: 4000,
  [AlertMode.warning]: 4000,
  [AlertMode.error]: 20000,
  [AlertMode.success]: 4000,
};
