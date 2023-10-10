import { ChangeDetectionStrategy, Component, Inject, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MAT_SNACK_BAR_DATA, MatSnackBarModule, MatSnackBarRef } from '@angular/material/snack-bar';

import { AlertComponent } from '../alert/alert.component';
import { AlertInterface } from '../alert/alert.interface';

@Component({
  selector: 'app-alert-wrap',
  exportAs: 'appAlertWrap',
  standalone: true,
  imports: [CommonModule, MatSnackBarModule, AlertComponent],
  templateUrl: './alert-wrap.component.html',
  styleUrls: ['./alert-wrap.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AlertWrapComponent {
  constructor(public snackBarRef: MatSnackBarRef<AlertWrapComponent>, @Inject(MAT_SNACK_BAR_DATA) public data: AlertInterface) {}

  // Performs the close on the snack bar.
  public doClose(): void {
    this.snackBarRef.dismissWithAction();
  }
}
