import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnChanges, OnInit, Output, 
  SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
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

  public controls1 = {
    nickname: new FormControl(null, []),
    email: new FormControl(null, []),
    password: new FormControl(null, []),
    avatar: new FormControl('', []),
    descript: new FormControl(null, []),
  };
  public formGroup1: FormGroup = new FormGroup(this.controls1);

  public controls2 = {
    password: new FormControl(null, []),
    new_password: new FormControl(null, []),
  };
  public formGroup2: FormGroup = new FormGroup(this.controls2);
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
  }

  ngOnInit(): void {
    this.fieldNicknameComp.getFormControl().markAsTouched();
    this.fieldEmailComp.getFormControl().markAsTouched();
  }
    
  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['profileDto']) {
      this.prepareForm1GroupByUserDto(this.profileDto);
      this.controls2.password.setValue(null);
      this.controls2.new_password.setValue(null);
      this.isRequiredPassword = false;
    }
    if (!!changes['isDisabledSubmit']) {
      if (this.isDisabledSubmit != this.formGroup1.disabled) {
        this.isDisabledSubmit ? this.formGroup1.disable() : this.formGroup1.enable();
        this.changeDetector.markForCheck();
      }
      if (this.isDisabledSubmit != this.formGroup2.disabled) {
        this.isDisabledSubmit ? this.formGroup2.disable() : this.formGroup2.enable();
        this.changeDetector.markForCheck();
      }
    }
  }
 
  // ** Public API **
  
  // ** Section "Udate profile" FormGroup1 **

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
      this.controls1.avatar.markAsDirty();
    } else {
      this.controls1.avatar.markAsPristine();
    }
  }

  public updateErrMsgsProfile(errMsgList: string[] = []): void {
    this.errMsgsProfile = errMsgList;
  }

  public saveProfile(): void {
    if (this.formGroup1.pristine || this.formGroup1.invalid) {
      return;
    }
    const nickname: string = this.controls1.nickname.value || '';
    const email: string = this.controls1.email.value || '';
    const descript: string = this.controls1.descript.value || '';

    const updateProfileFileDto: UpdateProfileFile = {
      nickname: (this.origProfileDto.nickname != nickname ? nickname : undefined),
      email: (this.origProfileDto.email != email ? email : undefined),
      // role?: string; // UserRole ["User","Admin"]
      descript: (this.origProfileDto.descript != descript ? descript : undefined),
      // theme?: string | undefined; // Default color theme. ["light","dark"]
      avatarFile: this.avatarFile,
    };
    let is_all_empty = Object.values(updateProfileFileDto).findIndex((value) => value != undefined) == -1;
    if (!is_all_empty) {
      this.updateProfile.emit(updateProfileFileDto);
    }
  }

  // ** Section "Set new password" FormGroup2 **

  public statePasswordField(passwordValue: string | null) {
    if (this.isRequiredPassword !== !!passwordValue) {
      this.isRequiredPassword = !!passwordValue;
    }
  }

  public setNewPassword(): void {
    if (this.controls2.password.pristine || this.controls2.password.invalid
     || this.controls2.new_password.pristine || this.controls2.new_password.invalid
     || !this.controls2.password.value || !this.controls2.new_password.value) {
       return;
    }
    const newPasswordProfileDto: NewPasswordProfileDto = {
        password: this.controls2.password.value, 
        newPassword: this.controls2.new_password.value
    };
    this.updatePassword.emit(newPasswordProfileDto);
  }

  public updateErrMsgsPassword(errMsgList: string[] = []): void {
    this.errMsgsPassword = errMsgList;
  }

  // ** Section "Delete Account" **

  public performDeleteAccount(): void {
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

    this.formGroup1.patchValue({
      nickname: profileDto.nickname,
      email: profileDto.email,
      descript: profileDto.descript,
      avatar: profileDto.avatar,
    });
    this.avatarFile = undefined;
    this.initIsAvatar = !!this.controls1.avatar.value;
     this.avatarView = profileDto.avatar || '';
     this.isAvatarOrig = !!this.avatarView;
  }
}
