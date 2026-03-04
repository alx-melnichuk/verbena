import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgForgotPassword } from './pg-forgot-password';

describe('PgForgotPassword', () => {
  let component: PgForgotPassword;
  let fixture: ComponentFixture<PgForgotPassword>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgForgotPassword]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgForgotPassword);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
