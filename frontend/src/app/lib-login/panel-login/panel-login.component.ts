import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, HostListener, Input, OnChanges, Output,
  SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { TranslateModule } from '@ngx-translate/core';

import { StrParams } from 'src/app/common/str-params';
import { ROUTE_FORGOT_PASSWORD, ROUTE_SIGNUP } from 'src/app/common/routes';
import { EMAIL_MAX_LENGTH, EMAIL_MIN_LENGTH } from 'src/app/components/field-email/field-email.component';
import { FieldNicknameComponent, NICKNAME_MAX_LENGTH, NICKNAME_MIN_LENGTH, NICKNAME_PATTERN
} from 'src/app/components/field-nickname/field-nickname.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';

@Component({
  selector: 'app-panel-login',
  exportAs: 'appPanelLogin',
  standalone: true,
  imports: [ CommonModule, RouterLink, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, TranslateModule,
    FieldNicknameComponent, FieldPasswordComponent,],
  templateUrl: './panel-login.component.html',
  styleUrls: ['./panel-login.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelLoginComponent implements OnChanges {
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgs: string[] = [];
  @Output()
  readonly login: EventEmitter<StrParams> = new EventEmitter();

  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }

  public linkSignup = ROUTE_SIGNUP;
  public linkForgotPassword = ROUTE_FORGOT_PASSWORD;

  public controls = {
    nickname: new FormControl<string | null>(null, []),
    email: new FormControl<string | null>(null, []),
    password: new FormControl<string | null>(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);

  public isEmail: boolean = false;
  public nicknameMinLen: number = NICKNAME_MIN_LENGTH;
  public nicknameMaxLen: number = NICKNAME_MAX_LENGTH;
  public nicknamePattern: string = NICKNAME_PATTERN;

  public emailMinLen: number = EMAIL_MIN_LENGTH;
  public emailMaxLen: number = EMAIL_MAX_LENGTH;

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

  public updateErrMsg(errMsgs: string[] = []): void {
    this.errMsgs = errMsgs;
  }

  public changeType(target: any ): void {
    this.isEmail = (target || { "value":"" }).value.indexOf('@') > -1;
  }

  public nicknameFocusout(): void {
    let value = this.controls.nickname.value || "";
    let clearedValue = value.trim();
    if (value != clearedValue) {
        this.controls.nickname.setValue(clearedValue);
    }
  }
}
