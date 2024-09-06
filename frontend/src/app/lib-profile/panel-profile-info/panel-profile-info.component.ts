import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnChanges, OnInit, Output, 
  SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { AbstractControl, FormControl, FormGroup, ReactiveFormsModule, ValidationErrors, ValidatorFn } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { IMAGE_VALID_FILE_TYPES, MAX_FILE_SIZE } from 'src/app/common/constants';
import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldEmailComponent    } from 'src/app/components/field-email/field-email.component';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { FieldImageAndUploadComponent } from 'src/app/components/field-image-and-upload/field-image-and-upload.component';
import { FieldNicknameComponent } from 'src/app/components/field-nickname/field-nickname.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { UniquenessCheckComponent } from 'src/app/components/uniqueness-check/uniqueness-check.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';

import { ProfileService } from '../profile.service';
import { NewPasswordProfileDto, ProfileDto, ProfileDtoUtil, UniquenessDto, UpdateProfileFile } from '../profile-api.interface';

export const PPI_DEBOUNCE_DELAY = 900;

@Component({
  selector: 'app-panel-profile-info',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, TranslateModule, FieldNicknameComponent,
    FieldEmailComponent, FieldPasswordComponent, FieldDescriptComponent, FieldFileUploadComponent, FieldImageAndUploadComponent,
     UniquenessCheckComponent],
  templateUrl: './panel-profile-info.component.html',
  styleUrls: ['./panel-profile-info.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelProfileInfoComponent implements OnInit, OnChanges {
  @Input()
  public profileDto: ProfileDto | null = null;
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgsProfile: string[] = [];
  @Input()
  public errMsgsPassword: string[] = [];
  @Input()
  public errMsgsAccount: string[] = [];
  
  @ViewChild(FieldNicknameComponent, { static: true })
  public fieldNicknameComp!: FieldNicknameComponent;
  @ViewChild(FieldEmailComponent, { static: true })
  public fieldEmailComp!: FieldEmailComponent;

  @Output()
  readonly updateProfile: EventEmitter<UpdateProfileFile> = new EventEmitter();
  @Output()
  readonly updatePassword: EventEmitter<NewPasswordProfileDto> = new EventEmitter();
  @Output()
  readonly deleteAccount: EventEmitter<void> = new EventEmitter();

  @HostBinding('class.global-scroll')
  public get classGlobalScrollVal(): boolean {
    return true;
  }

  public cntlsProfile = {
    nickname: new FormControl(null, []),
    email: new FormControl(null, []),
    password: new FormControl(null, []),
    avatar: new FormControl('', []),
    descript: new FormControl(null, []),
  };
  public formGroupProfile: FormGroup = new FormGroup(this.cntlsProfile);

  public cntlsPassword = {
    password: new FormControl(null, []),
    new_password: new FormControl(null, []),
  };
  public formGroupPassword: FormGroup = new FormGroup(this.cntlsPassword);
  public isRequiredPassword: boolean = false;

  public debounceDelay: number = PPI_DEBOUNCE_DELAY;
  // Avatar Image Options
  public maxSize = MAX_FILE_SIZE;
  public validTypes = IMAGE_VALID_FILE_TYPES;
  public avatarFile: File | null | undefined;
  public initIsAvatar: boolean = false; // original has an avatar.
  public avatarView: string = '';
  public isAvatarOrig: boolean = false; // original has an avatar.
  
  private origProfileDto: ProfileDto = ProfileDtoUtil.create();

  constructor(
    private changeDetector: ChangeDetectorRef,
    private translate: TranslateService,
    private dialogService: DialogService,
    private profileService: ProfileService,
  ) {
    this.formGroupPassword.setValidators(this.validatorsForPassword());
  }

  ngOnInit(): void {
    this.fieldNicknameComp.getFormControl().markAsTouched();
    this.fieldEmailComp.getFormControl().markAsTouched();
  }
    
  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['profileDto']) {
      this.prepareForm1GroupByUserDto(this.profileDto);
      this.cntlsPassword.password.setValue(null);
      this.cntlsPassword.new_password.setValue(null);
      this.isRequiredPassword = false;
    }
    if (!!changes['isDisabledSubmit']) {
      if (this.isDisabledSubmit != this.formGroupProfile.disabled) {
        this.isDisabledSubmit ? this.formGroupProfile.disable() : this.formGroupProfile.enable();
        this.changeDetector.markForCheck();
      }
      if (this.isDisabledSubmit != this.formGroupPassword.disabled) {
        this.isDisabledSubmit ? this.formGroupPassword.disable() : this.formGroupPassword.enable();
        this.changeDetector.markForCheck();
      }
    }
  }
 
  // ** Public API **
  
  // ** Section: Update profile (formGroupProfile) **

  public checkUniquenessNickname = (nickname: string | null | undefined): Promise<boolean> => {
    if (!nickname || this.origProfileDto.nickname.toLowerCase() == nickname.toLowerCase()) {
      return Promise.resolve(true);
    }
    return this.profileService.uniqueness(nickname, '').then((response) => response == null || (response as UniquenessDto).uniqueness);
  }

  public checkUniquenessEmail = (email: string | null | undefined): Promise<boolean> => {
    if (!email || this.origProfileDto.email.toLowerCase() == email.toLowerCase()) {
      return Promise.resolve(true);
    }
    return this.profileService.uniqueness('', email).then((response) => response == null || (response as UniquenessDto).uniqueness);
  }

  public addAvatarFile(file: File): void {
    this.avatarFile = file;
  }
  public deleteAvatarFile(): void {
    this.avatarFile = (!!this.initIsAvatar ? null : undefined);
    if (!!this.initIsAvatar) {
      this.cntlsProfile.avatar.markAsDirty();
    } else {
      this.cntlsProfile.avatar.markAsPristine();
    }
  }

  public updateErrMsgsProfile(errMsgList: string[] = []): void {
    this.errMsgsProfile = errMsgList;
  }

  public saveProfile(formGroup: FormGroup): void {
    const cntlNickname = formGroup.get('nickname');
    const cntlEmail = formGroup.get('email');
    const cntlDescript = formGroup.get('descript');
    if (formGroup.pristine ||  formGroup.invalid || !cntlNickname || !cntlEmail || !cntlDescript) {
      return;
    }
    const nickname: string = cntlNickname.value || '';
    const email: string = cntlEmail.value || '';
    const descript: string = cntlDescript.value || '';

    const updateProfileFileDto: UpdateProfileFile = {
      nickname: (this.origProfileDto.nickname != nickname ? nickname : undefined),
      email: (this.origProfileDto.email != email ? email : undefined),
      descript: (this.origProfileDto.descript != descript ? descript : undefined),
      // theme?: string | undefined; // Default color theme. ["light","dark"]
      avatarFile: this.avatarFile,
    };
    let is_all_empty = Object.values(updateProfileFileDto).findIndex((value) => value != undefined) == -1;
    if (!is_all_empty) {
      this.updateProfile.emit(updateProfileFileDto);
    }
  }

  // ** Section: Set new password (formPassword) **

  public validatorsForPassword(): ValidatorFn {
    return (control: AbstractControl): ValidationErrors | null => {
      const formGroup = control as FormGroup; 
      const cntlPassword = formGroup.get('password');
      const cntlNewPassword = formGroup.get('new_password');
      if (cntlPassword?.pristine) {
        return { pristine: 'password' };
      }
      if (cntlPassword?.invalid) {
        return { invalid: 'password' };
      }
      if (cntlNewPassword?.pristine) {
        return { pristine: 'new_password' };
      }
      if (cntlNewPassword?.invalid) {
        return { invalid: 'new_password' };
      }
      const passwordValue = cntlPassword?.value || '';
      const newPasswordValue = cntlNewPassword?.value || '';
      if (!!passwordValue && !!newPasswordValue && passwordValue == newPasswordValue) {
        return { new_password_equal_to_old_value: true };
      }
      return null;
    };
  }

  public statePassword(passwordValue: string | null) {
    if (this.isRequiredPassword !== !!passwordValue) {
      this.isRequiredPassword = !!passwordValue;
    }
  }

  public checkPassword(formGroup: FormGroup): void {
    if (formGroup.errors != null && formGroup.errors['new_password_equal_to_old_value']) {
      this.errMsgsPassword.push('Validation.new_password:equal_to_old_value');
    }
  }

  public setNewPassword(formGroup: FormGroup): void {
    const cntlPassword = formGroup.get('password');
    const cntlNewPassword = formGroup.get('new_password');
    if (formGroup.pristine ||  formGroup.invalid || !cntlPassword || !cntlNewPassword) {
      return;
    }
    const newPasswordProfileDto: NewPasswordProfileDto = {
        password: cntlPassword.value,
        newPassword: cntlNewPassword.value
    };
    this.updatePassword.emit(newPasswordProfileDto);
  }

  public updateErrMsgsPassword(errMsgList: string[] = []): void {
    this.errMsgsPassword = errMsgList;
  }

  // ** Section "Delete Account" **

  public removeAccount(): void {
    const title = this.translate.instant('profile.dialog_title_question_account');
    const nickname = this.profileDto?.nickname || '';
    const appName = this.translate.instant('app.name');
    const message = this.translate.instant('profile.dialog_message_question_account', { nickname, appName: appName });
    this.dialogService.openConfirmation(message, title, 'buttons.no', 'buttons.yes').then((respose) => {
      if (!!respose) {
        this.deleteAccount.emit();
      }
    });
  }

  // ** Private API **

  private prepareForm1GroupByUserDto(profileDto: ProfileDto | null): void {
    if (!profileDto) {
      return;
    }
    this.origProfileDto = { ...profileDto };
    Object.freeze(this.origProfileDto);

    this.formGroupProfile.patchValue({
      nickname: profileDto.nickname,
      email: profileDto.email,
      descript: profileDto.descript,
      avatar: profileDto.avatar,
    });
    this.avatarFile = undefined;
    this.initIsAvatar = !!this.cntlsProfile.avatar.value;
     this.avatarView = profileDto.avatar || '';
     this.isAvatarOrig = !!this.avatarView;
  }
}
