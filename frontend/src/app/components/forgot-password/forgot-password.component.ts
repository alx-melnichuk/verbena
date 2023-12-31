import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostListener, Input, OnChanges, Output,
  SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { TranslateModule } from '@ngx-translate/core';

import { StrParams } from '../../common/str-params';
import { FieldEmailComponent } from '../field-email/field-email.component';
import { ROUTE_LOGIN } from 'src/app/common/routes';

@Component({
  selector: 'app-forgot-password',
  standalone: true,
  imports: [
    CommonModule,
    RouterLink,
    TranslateModule,
    ReactiveFormsModule,
    MatButtonModule,
    MatFormFieldModule,
    MatInputModule,
    FieldEmailComponent,
  ],
  templateUrl: './forgot-password.component.html',
  styleUrls: ['./forgot-password.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ForgotPasswordComponent implements OnChanges {
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList: string[] = [];
  @Output()
  readonly resend: EventEmitter<StrParams> = new EventEmitter();

  public linkLogin = ROUTE_LOGIN;

  public controls = {
    email: new FormControl<string | null>(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);

  constructor(private changeDetector: ChangeDetectorRef) {}

  @HostListener('document:keypress', ['$event'])
  public keyEvent(event: KeyboardEvent): void {
    if (event.code === 'Enter') {
      this.doResend();
    }
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isDisabledSubmit']) {
      if (this.isDisabledSubmit != this.formGroup.disabled) {
        this.isDisabledSubmit ? this.formGroup.disable() : this.formGroup.enable();
        this.changeDetector.markForCheck();
      }
    }
  }

  // ** Public API **

  public doResend(): void {
    if (this.formGroup.invalid || this.isDisabledSubmit) {
      return;
    }
    const email = this.controls.email.value;
    this.resend.emit({ email });
  }

  public updateErrMsg(errMsgList: string[] = []): void {
    this.errMsgList = errMsgList;
  }

}
