import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgAbout } from './pg-about';

describe('PgAbout', () => {
  let component: PgAbout;
  let fixture: ComponentFixture<PgAbout>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgAbout]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgAbout);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
