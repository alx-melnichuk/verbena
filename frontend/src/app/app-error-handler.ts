import { ErrorHandler, Injectable } from '@angular/core';

@Injectable()
export class AppErrorHandler implements ErrorHandler {
  // Handling the error "Loading chunk [\d]+ failed"
  public handleError(error: any): void {
    const chunkFailedMessage = /Loading chunk [\d]+ failed/;
    if (chunkFailedMessage.test(error?.message)) {
      window.location.reload();
    }
  }
}