import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, EventEmitter, HostListener, Input, Output, ViewEncapsulation } from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { StrParams } from '../../common/str-params';
import { FieldPasswordComponent } from '../field-password/field-password.component';
import { FieldEmailComponent } from '../field-email/field-email.component';
import { FieldNicknameComponent } from '../field-nickname/field-nickname.component';

const MIN_NICKNAME_LENGTH = 3;
const MAX_NICKNAME_LENGTH = 10; // 50
const PATTERN_NICKNAME = '^[a-zA-Z0-9]+$';

@Component({
  selector: 'app-signup',
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
    FieldNicknameComponent,
    FieldPasswordComponent,
  ],
  templateUrl: './signup.component.html',
  styleUrls: ['./signup.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SignupComponent {
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList: string[] = [];
  @Output()
  readonly signup: EventEmitter<StrParams> = new EventEmitter();

  public linkLogin = 'login'; // ROUTE_LOGIN;
  // public linkSignup = ROUTE_SIGNUP;
  public controls = {
    nickname: new FormControl<string | null>(null, []),
    email: new FormControl<string | null>(null, []),
    password: new FormControl<string | null>(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);

  public minLenNickname = MIN_NICKNAME_LENGTH;
  public maxLenNickname = MAX_NICKNAME_LENGTH;
  public patternNickname = PATTERN_NICKNAME;

  // public isNicknameHasEmail = false;
  // public linkForgotPassword = ROUTE_CONFIRMATION_FORGOT_PASSWORD;

  // public minLenPassword = MIN_PASSWORD_LENGTH;
  // public maxLenPassword = MAX_PASSWORD_LENGTH;
  // public patternPassword = PATTERN_PASSWORD;

  constructor(public translate: TranslateService) {}

  @HostListener('document:keypress', ['$event'])
  public keyEvent(event: KeyboardEvent): void {
    if (event.code === 'Enter') {
      this.doLogin();
    }
  }

  // ** Public API **

  public doLogin(): void {
    if (this.formGroup.invalid || this.isDisabledSubmit) {
      return;
    }
    const nickname = this.controls.nickname.value;
    const password = this.controls.password.value;
    const email = this.controls.email.value;
    this.signup.emit({ nickname, email, password });
  }

  public updateErrMsg(errMsgList: string[] = []): void {
    this.errMsgList = errMsgList;
  }
}
