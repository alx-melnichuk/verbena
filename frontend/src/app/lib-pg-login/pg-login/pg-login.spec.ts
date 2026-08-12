import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgLogin } from './pg-login';

describe('PgLogin', () => {
  let component: PgLogin;
  let fixture: ComponentFixture<PgLogin>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgLogin]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgLogin);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
