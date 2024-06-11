import {
  ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { TranslateModule } from '@ngx-translate/core';

import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldEmailComponent } from 'src/app/components/field-email/field-email.component';
import { FieldNicknameComponent } from 'src/app/components/field-nickname/field-nickname.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { UserDto, UserDtoUtil }   from 'src/app/lib-user/user-api.interface';
import { IMAGE_VALID_FILE_TYPES, MAX_FILE_SIZE } from 'src/app/common/constants';

@Component({
  selector: 'app-panel-profile-info',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, MatButtonModule, TranslateModule, FieldNicknameComponent, FieldEmailComponent,
    FieldPasswordComponent, FieldDescriptComponent, FieldFileUploadComponent],
  templateUrl: './panel-profile-info.component.html',
  styleUrls: ['./panel-profile-info.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelProfileInfoComponent implements OnChanges {
  @Input()
  public userInfo: UserDto | null = null;
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList: string[] = [];
  
  public controls = {
    avatar: new FormControl(null, []),
    nickname: new FormControl(null, []),
    email: new FormControl(null, []),
    password: new FormControl(null, []),
    descript: new FormControl(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);

  public maxFileSize = MAX_FILE_SIZE;
  public validFileTypes = IMAGE_VALID_FILE_TYPES;
  public addedLogoView: string = '';
  private origUserDto: UserDto = UserDtoUtil.create();

  constructor() {
  }
    
  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['userInfo']) {
      this.prepareFormGroupByUserDto(this.userInfo);
    }
  }
 
  // ** Public API **
  
  public addFile(file: File): void {
    // this.addedLogoFile = file;
    // this.controls.logo.setValue(file.name);
    // this.controls.logo.markAsDirty();
  }

  public readFile(buffFile: string[]): void {
    if (buffFile.length > 0) {
      this.addedLogoView = buffFile[1];
    //   this.changeDetectorRef.markForCheck();
    }
  }

  public deleteFileLogo(): void {
    // this.addedLogoFile = (!!this.origLogo ? null : undefined);
    this.addedLogoView = '';
    // this.controls.logo.setValue(null);
    // if (!!this.origLogo) {
    //   this.controls.logo.markAsDirty();
    // } else {
    //   this.controls.logo.markAsPristine();
    // }
  }

  public updateErrMsg(errMsgList: string[] = []): void {
    this.errMsgList = errMsgList;
  }

  public saveProfile(): void {
    console.log(`saveProfile();`);
  }

  public cancelProfile(): void {
    console.log(`cancelProfile();`);
  }

  // ** Private API **

  private prepareFormGroupByUserDto(userInfo: UserDto | null): void {
    if (!userInfo) {
      return;
    }
    this.origUserDto = { ...userInfo };
    Object.freeze(this.origUserDto);

    this.formGroup.patchValue({
      avatar: '',
      nickname: userInfo.nickname,
      email: userInfo.email,
      password: userInfo.password,
      descript: 'Description of a beautiful trip 2024 to greece 6 - E.Allen',
    });
  }

}
