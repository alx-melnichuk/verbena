import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgStream } from './pg-stream';

describe('PgStream', () => {
  let component: PgStream;
  let fixture: ComponentFixture<PgStream>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgStream]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgStream);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
