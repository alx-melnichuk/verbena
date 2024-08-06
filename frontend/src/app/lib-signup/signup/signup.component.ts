import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
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

import { FieldEmailComponent } from 'src/app/components/field-email/field-email.component';
import { FieldNicknameComponent } from 'src/app/components/field-nickname/field-nickname.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { ROUTE_LOGIN } from 'src/app/common/routes';
import { StrParams } from 'src/app/common/str-params';
import { UniquenessCheckComponent } from 'src/app/components/uniqueness-check/uniqueness-check.component';
import { ProfileService } from 'src/app/lib-profile/profile.service';
import { UniquenessDto } from 'src/app/lib-profile/profile-api.interface';

export const SG_DEBOUNCE_DELAY = 900;

@Component({
  selector: 'app-signup',
  standalone: true,
  imports: [ CommonModule, RouterLink, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, TranslateModule,
    FieldEmailComponent, FieldNicknameComponent, FieldPasswordComponent, UniquenessCheckComponent],
  templateUrl: './signup.component.html',
  styleUrls: ['./signup.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SignupComponent implements OnChanges {
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList: string[] = [];
  @Output()
  readonly signup: EventEmitter<StrParams> = new EventEmitter();

  public linkLogin = ROUTE_LOGIN;
  public debounceDelay: number = SG_DEBOUNCE_DELAY;

  public controls = {
    nickname: new FormControl<string | null>(null, []),
    email: new FormControl<string | null>(null, []),
    password: new FormControl<string | null>(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);

  constructor(
    private changeDetector: ChangeDetectorRef,
    private profileService: ProfileService,
  ) {
  }

  @HostListener('document:keypress', ['$event'])
  public keyEvent(event: KeyboardEvent): void {
    if (event.code === 'Enter') {
      this.doSignup();
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

  public checkUniquenessNickname = (nickname: string | null | undefined): Promise<boolean> => {
    if (!nickname) {
      return Promise.resolve(true);
    }
    return this.profileService.uniqueness(nickname, '').then((response) => response == null || (response as UniquenessDto).uniqueness);
  }

  public checkUniquenessEmail = (email: string | null | undefined): Promise<boolean> => {
    if (!email) {
      return Promise.resolve(true);
    }
    return this.profileService.uniqueness('', email).then((response) => response == null || (response as UniquenessDto).uniqueness);
  }

  public doSignup(): void {
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
