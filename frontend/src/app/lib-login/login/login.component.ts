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

import { EMAIL_MAX_LENGTH, EMAIL_MIN_LENGTH } from 'src/app/components/field-email/field-email.component';
import { FieldNicknameComponent, NICKNAME_MAX_LENGTH, NICKNAME_MIN_LENGTH, NICKNAME_PATTERN
} from 'src/app/components/field-nickname/field-nickname.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { StrParams } from 'src/app/common/str-params';
import { ROUTE_FORGOT_PASSWORD, ROUTE_SIGNUP } from 'src/app/common/routes';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [ CommonModule, RouterLink, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, TranslateModule,
    FieldNicknameComponent, FieldPasswordComponent,],
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LoginComponent implements OnChanges {
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList: string[] = [];
  @Output()
  readonly login: EventEmitter<StrParams> = new EventEmitter();

  public linkSignup = ROUTE_SIGNUP;
  public linkForgotPassword = ROUTE_FORGOT_PASSWORD;

  public nicknameMinLen: number = NICKNAME_MIN_LENGTH;
  public nicknameMaxLen: number = NICKNAME_MAX_LENGTH;
  public nicknamePattern: string = NICKNAME_PATTERN;

  public emailMinLen: number = EMAIL_MIN_LENGTH;
  public emailMaxLen: number = EMAIL_MAX_LENGTH;

  public controls = {
    nickname: new FormControl<string | null>(null, []),
    email: new FormControl<string | null>(null, []),
    password: new FormControl<string | null>(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);
  public isEmail: boolean = false;

  constructor(private changeDetector: ChangeDetectorRef) {}

  @HostListener('document:keypress', ['$event'])
  public keyEvent(event: KeyboardEvent): void {
    if (event.code === 'Enter') {
      this.doLogin();
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

  public doLogin(): void {
    if (this.formGroup.invalid || this.isDisabledSubmit) {
      return;
    }
    const nickname = this.controls.nickname.value;
    const password = this.controls.password.value;
    this.login.emit({ nickname, password });
  }

  public changeType(target: any ): void {
    this.isEmail = (target || { "value":"" }).value.indexOf('@') > -1;
  }

  public updateErrMsg(errMsgList: string[] = []): void {
    this.errMsgList = errMsgList;
  }

  public nicknameFocusout(): void {
    let value = this.controls.nickname.value || "";
    let clearedValue = value.trim();
    if (value != clearedValue) {
        this.controls.nickname.setValue(clearedValue);
    }
  }
}
