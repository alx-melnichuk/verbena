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
    [AlertMode.comment]: 6000,
    [AlertMode.info]: 6000,
    [AlertMode.warning]: 6000,
    [AlertMode.error]: 30000,
    [AlertMode.success]: 6000,
};
