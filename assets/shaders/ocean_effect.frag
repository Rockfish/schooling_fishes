// Jaybird's Simple Water Caustic Pattern
// https://www.shadertoy.com/view/3d3yRj
//
// Adapted from https://www.shadertoy.com/view/Ws23RD

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{

	vec2 p = (-iResolution.xy + 2.0*fragCoord) / iResolution.y;

    // Uncomment, and then set any settings value to either prevX/Y to preview it with the mouse
    float prevX  = (iMouse.x/iResolution.x)*4.0-2.0;
    float prevY  = (iMouse.y/iResolution.y)*4.0-2.0;

    // Settings
    float invertY    = -1.0; // 1.0 = invert y axis (application), -1.0= don't invert (shadertoy)
    float yaw        = -0.03;  // Rotate camera on z axis (like saying no with your head).
    float pitch      = 0.6;  // Rotate camera like saying yes with your head
    float roll       = 0.0;  // Rotate camera like putting your head to your shoulder
    float height     = 2.0;  // Height of the room, BUT also changes the pitch downwards.
    float fov        = 1.0;  // Basically zoom, comes with perspective distortion too.

    float scale      = 8.0; // Size of the rays (also changes the speed)
    float speed      = 0.16; // How quickly the rays dance
    float brightness = 1.7;  // Smaller = brighter, more intense rays
    float contrast   = 2.0;  // Difference between ray and darkness. Smaller = more grey.
    float multiply   = 0.2;  // Alpha/transparency and colour intensity of final result
    vec3  rayColour  = vec3(1.0,0.964,0.690); // rgb colour of rays

    // Move the camera
    float offsetX    = -prevX*15.0;
    float offsetY    = prevY*15.0;

    // Camera matrix complicated maths stuff
    vec3 ww = normalize(invertY*vec3(yaw, height, pitch));
    vec3 uu = normalize(cross(ww, vec3(roll, 1.0, 0.0)));
    vec3 vv = normalize(cross(uu,ww));
	vec3 rd = p.x*uu + p.y*vv + fov*ww;	// view ray
    vec3 pos = -ww + rd*(ww.y/rd.y);	// raytrace plane

    pos.y = iTime * speed;				// animate noise slice
    pos *= scale;				        // tiling frequency

    // Apply the offsets to camera position
    pos.x += offsetX;
    pos.z += offsetY;

    // Generate some noise
    vec4 noise = snoise( pos );

    // Offset it and regenerate x2
    pos -= 0.07*noise.xyz;
    noise = snoise( pos );

    pos -= 0.07*noise.xyz;
    noise = snoise( pos );

    // Calculate intensity of this pixel
    float intensity = exp(noise.w*contrast - brightness);

    // Generate a lovely warm oceany gradient
    vec4 c = vec4(
        234.0/255.0-(fragCoord.y/iResolution.y)*0.7,
        235.0/255.0-(fragCoord.y/iResolution.y)*0.4,
        166.0/255.0-(fragCoord.y/iResolution.y)*0.1,1.0);

    // Generate final rgba of this pixel
	fragColor = c + vec4(rayColour * multiply * intensity, intensity);
}