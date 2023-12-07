import { ChangeDetectionStrategy, Component, Inject, OnInit, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatDialogRef, MAT_DIALOG_DATA, MatDialogModule } from '@angular/material/dialog';
import { TranslateService } from '@ngx-translate/core';
import { MatButtonModule } from '@angular/material/button';

export interface ConfirmationData {
  title?: string;
  message?: string;
  messageHtml?: string;
  btnNameCancel?: string;
  btnNameAccept?: string;
}

@Component({
  selector: 'app-confirmation',
  exportAs: 'appConfirmation',
  standalone: true,
  imports: [CommonModule, MatDialogModule, MatButtonModule],
  templateUrl: './confirmation.component.html',
  styleUrls: ['./confirmation.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ConfirmationComponent implements OnInit {
  public title = 'dialog.confirmation';
  public message: string | null = null;
  public messageHtml: string | null = null;
  public btnNameCancel: string | null = 'Cancel';
  public btnNameAccept: string | null = 'Accept';

  constructor(
    public dialogRef: MatDialogRef<ConfirmationComponent>,
    @Inject(MAT_DIALOG_DATA) public data: ConfirmationData,
    private translate: TranslateService
  ) {
    this.title = this.getTranslate(data.title || this.title);
    this.message = this.getTranslate(data.message || this.message);
    this.messageHtml = this.getTranslate(data.messageHtml || this.messageHtml);
    this.btnNameCancel = data.btnNameCancel != null ? this.getTranslate(data.btnNameCancel || this.btnNameCancel) : null;
    this.btnNameAccept = data.btnNameAccept != null ? this.getTranslate(data.btnNameAccept || this.btnNameAccept) : null;
  }

  ngOnInit(): void {}

  public cancel(): void {
    this.dialogRef.close();
  }

  public accept(): void {
    this.dialogRef.close(true);
  }

  private getTranslate(name: string | null): string {
    return !!name ? this.translate.instant(name) : name;
  }
}
