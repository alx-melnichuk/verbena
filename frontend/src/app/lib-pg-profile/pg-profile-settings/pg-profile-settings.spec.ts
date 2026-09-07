import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgProfileSettings } from './pg-profile-settings';

describe('PgProfileSettings', () => {
  let component: PgProfileSettings;
  let fixture: ComponentFixture<PgProfileSettings>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgProfileSettings]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgProfileSettings);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
