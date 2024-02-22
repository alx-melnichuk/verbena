import { ComponentType } from '@angular/cdk/portal';
import { Injectable } from '@angular/core';
import { MatDialog, MatDialogConfig } from '@angular/material/dialog';

import { ConfirmationComponent, ConfirmationData } from './confirmation/confirmation.component';

@Injectable({
  providedIn: 'root',
})
export class DialogService {
  constructor(private dialog: MatDialog) {}

  // ** Public API **

  public openConfirmation(message: string, title?: string, btnNameCancel?: string | null, btnNameAccept?: string | null): Promise<unknown> {
    const confirmationData: ConfirmationData = { title, message };
    if (!!btnNameCancel) {
      confirmationData.btnNameCancel = btnNameCancel;
    }
    if (!!btnNameAccept) {
      confirmationData.btnNameAccept = btnNameAccept;
    }
    return this.openComponentExt(ConfirmationComponent, confirmationData);
  }

  public openComponent(component: ComponentType<unknown>, dataParams: any): void {
    const dialogConfig = new MatDialogConfig();
    // Custom class for the overlay pane.
    dialogConfig.panelClass = ['app-modal-panel', 'large'];
    // Whether the dialog has a backdrop.
    dialogConfig.hasBackdrop = true;
    // Custom class for the backdrop.
    dialogConfig.backdropClass = 'app-modal-backdrop';
    // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
    dialogConfig.disableClose = false;
    dialogConfig.data = dataParams;

    this.dialog.open(component, dialogConfig);
  }

  public openComponentExt(component: ComponentType<unknown>, dataParams: any): Promise<unknown> {
    const dialogConfig = new MatDialogConfig();
    // Custom class for the overlay pane.
    dialogConfig.panelClass = ['app-modal-panel', 'large'];
    // Whether the dialog has a backdrop.
    dialogConfig.hasBackdrop = true;
    // Custom class for the backdrop.
    dialogConfig.backdropClass = 'app-modal-backdrop';
    // Whether the user can use escape or clicking on the backdrop to close the modal. disableClose?: boolean;
    dialogConfig.disableClose = false;
    dialogConfig.data = dataParams;

    const dialogRef = this.dialog.open(ConfirmationComponent, dialogConfig);
    // const answer = new Promise((resolve, reject) => {
    //   dialogRef.afterClosed().subscribe(result => {
    //     if (!!result) { resolve(result); } else { reject(); }
    //   });
    // });
    // return answer;
    return dialogRef.afterClosed().toPromise();
  }
}
